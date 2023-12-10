pub struct Parser<'a> {
    data: &'a [u8],
}


impl<'a> Parser<'a> {
    /// Construct a new parser for a slice of `u8`'s
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Take a `u8` from the slice.
    #[must_use]
    pub fn take_u8(&mut self) -> Option<u8> {
        if self.data.is_empty() {
            None
        } else {
            let result = self.data[0];
            self.data = &self.data[1..];

            Some(result)
        }
    }

    /// Take a `u16` from the slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is of the wrong size.
    #[must_use]
    pub fn take_u16(&mut self) -> Option<u16> {
        if self.data.len() >= 2 {
            let result = u16::from_le_bytes(self.data[0..2].try_into().unwrap());
            self.data = &self.data[2..];

            Some(result)
        } else {
            None
        }
    }

    /// Take a `u32` from the slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is of the wrong size.
    #[must_use]
    pub fn take_u32(&mut self) -> Option<u32> {
        if self.data.len() >= 4 {
            let result = u32::from_le_bytes(self.data[0..4].try_into().unwrap());
            self.data = &self.data[4..];

            Some(result)
        } else {
            None
        }
    }

    /// Take a `u64` from the slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is of the wrong size.
    #[must_use]
    pub fn take_u64(&mut self) -> Option<u64> {
        if self.data.len() >= 8 {
            let result = u64::from_le_bytes(self.data[0..8].try_into().unwrap());
            self.data = &self.data[8..];

            Some(result)
        } else {
            None
        }
    }

    /// Take a `u128` from the slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is of the wrong size.
    #[must_use]
    pub fn take_u128(&mut self) -> Option<u128> {
        if self.data.len() >= 16 {
            let result = u128::from_le_bytes(self.data[0..16].try_into().unwrap());
            self.data = &self.data[16..];

            Some(result)
        } else {
            None
        }
    }

    /// Take an array of `u8`'s of a given length.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is of the wrong size.
    pub fn take_u8_array<const L: usize>(&mut self) -> Option<[u8; L]> {
        if self.data.len() >= L {
            let result = self.data[0..L].try_into().unwrap();
            self.data = &self.data[L..];

            Some(result)
        } else {
            None
        }
    }

    /// Take an array of `u32`'s of a given length.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is of the wrong size.
    pub fn take_u32_array<const L: usize>(&mut self) -> Option<[u32; L]> {
        if self.data.len() >= L {
            let mut result = [0; L];
            for slot in result.iter_mut() {
                *slot = self.take_u32().unwrap();
            }

            Some(result)
        } else {
            None
        }
    }

    /// Skip a certain number of bytes
    pub fn skip(&mut self, length: usize) -> Option<()> {
        if self.data.len() >= length {
            self.data = &self.data[length..];

            Some(())
        } else {
            None
        }
    }

    /// Returns true if the buffer is empty
    #[must_use]
    pub const fn empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<'a> core::iter::Iterator for Parser<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.take_u8()
    }
}
