pub struct OsStrRef<'a> {
    data: &'a [u8]
}

impl<'a> OsStrRef<'a> {
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self {
            data
        }
    }
}

impl<'a> core::fmt::Display for OsStrRef<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for byte in self.data {
            if *byte < 0x80 {
                write!(f, "{}", *byte as char)?;
            }
            else {
                write!(f, "�")?;
            }
        }

        Ok(())
    }
}