/// Collection of locks stored together in a bitmap
#[allow(clippy::module_name_repetitions)]
pub struct BitmapLock {
    bitmap: &'static [core::sync::atomic::AtomicU64],
    length: usize,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmapError {
    RangeOutOfBounds {
        start: usize,
        end: usize,
        length: usize,
    },
    UnableToAllocate {
        length: usize,
    },
}

impl BitmapLock {
    /// Construct a new, empty `BitmapLock`
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bitmap: &[],
            length: 0,
        }
    }

    /// Construct a new `BitmapLock` from a buffer of `AtomicU64`s and a length
    pub const fn from_data(
        bitmap: &'static [core::sync::atomic::AtomicU64],
        length: usize,
    ) -> Self {
        let use_length = if bitmap.len() * 64 >= length {
            bitmap.len() * 64
        } else {
            length
        };

        Self {
            bitmap,
            length: use_length,
        }
    }

    fn try_set_in_entry(&self, entry_index: usize, mask: u64) -> bool {
        let read = self.bitmap[entry_index].fetch_or(mask, core::sync::atomic::Ordering::AcqRel);

        // If there were bits set that overlap our write, we need to leave those bits intact, and remove the excess bits
        if read & mask != 0 {
            let excess = mask & (!read);
            self.clear_in_entry(entry_index, excess);
        }

        // If none of the bits which are set in the mask were set when we wrote to the entry, we were the ones to acquire those bits.
        read & mask == 0
    }

    /// Attempts to set a sequence of `count` bits starting at *bit* index `index`. Note that any index returned will
    /// be less than `length`.
    ///
    /// # Errors
    ///
    /// This function will return an error if `count` bits after `index` does not fit within the bitmap.
    pub fn try_set(&self, index: usize, count: usize) -> Result<bool, BitmapError> {
        if index + count >= self.length {
            Err(BitmapError::RangeOutOfBounds {
                start: index,
                end: index + count,
                length: self.length,
            })
        } else {
            // Compute the entry index
            let entry_index_start = index / 64;
            let offset_into_entry = index % 64;

            let bit_count = count.min(64 - offset_into_entry);

            #[allow(clippy::cast_possible_truncation)] // `bit_count` is less than or equal to 64
            let this_mask = if bit_count < 64 {
                (1u64.wrapping_shl(bit_count as u32)).wrapping_sub(1) << offset_into_entry
            } else {
                u64::MAX
            };

            if self.try_set_in_entry(entry_index_start, this_mask) {
                let next_count = count - bit_count;
                if next_count > 0 {
                    if self.try_set(index + bit_count, count - bit_count)? {
                        Ok(true)
                    } else {
                        // If a later set failed, we need to clear the bits we have already set
                        self.clear(index, bit_count)?;

                        Ok(false)
                    }
                } else {
                    Ok(true)
                }
            } else {
                Ok(false)
            }
        }
    }

    fn clear_in_entry(&self, entry_index: usize, mask: u64) {
        let result =
            self.bitmap[entry_index].fetch_and(!mask, core::sync::atomic::Ordering::AcqRel);

        if result & mask != mask {
            warn!("Attempted to clear bits with mask {:x} at entry {} in bitmap lock, some bits were already cleared");
        }
    }

    /// Clears a sequence of `count` bits starting at *bit* index `index`.
    ///
    /// # Errors
    ///
    /// This function will return an error if `count` bits after index does not fit within the bitmap.
    pub fn clear(&self, index: usize, count: usize) -> Result<(), BitmapError> {
        if index + count >= self.length {
            Err(BitmapError::RangeOutOfBounds {
                start: index,
                end: index + count,
                length: self.length,
            })
        } else {
            // Compute the entry index
            let entry_index_start = index / 64;
            let offset_into_entry = index % 64;

            let bit_count = count.min(64 - offset_into_entry);

            #[allow(clippy::cast_possible_truncation)] // `bit_count` is less than or equal to 64
            let this_mask = if bit_count < 64 {
                (1u64.wrapping_shl(bit_count as u32)).wrapping_sub(1) << offset_into_entry
            } else {
                u64::MAX
            };

            self.clear_in_entry(entry_index_start, this_mask);
            let next_count = count - bit_count;
            if next_count > 0 {
                self.clear(index + bit_count, count - bit_count)?;
            }

            Ok(())
        }
    }

    /// Request a sequence of `count` bits to be locked, returning the *bit* index of the first lock if one is found.
    /// The index returned will be less than `length`.
    ///
    /// # Errors
    ///
    /// This function will return an error if it was unable to allocate `count` bits in the bitmap.
    pub fn reserve_sequence(&self, count: usize) -> Result<usize, BitmapError> {
        // Loop over the entries in the table
        for entry_index in 0..self.bitmap.len() {
            let mut mask_off_impossible_indexes =
                self.bitmap[entry_index].load(core::sync::atomic::Ordering::Relaxed);

            while mask_off_impossible_indexes != u64::MAX {
                let bit_index = mask_off_impossible_indexes.trailing_ones();
                let bit_offset = bit_index as usize;

                let index = entry_index * 64 + bit_offset;

                // Note that the index returned must be less than `length`.
                match self.try_set(index, count) {
                    Ok(true) => {
                        return Ok(index);
                    }
                    Err(BitmapError::RangeOutOfBounds { .. }) => {
                        break;
                    }
                    _ => {}
                }

                mask_off_impossible_indexes |= 1 << bit_index;
            }
        }

        Err(BitmapError::UnableToAllocate { length: count })
    }
}
