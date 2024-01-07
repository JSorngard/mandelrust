use core::fmt;
use core::num::{NonZeroU32, NonZeroUsize, TryFromIntError};
/// A struct containing a resolution that is known
/// to fit in both a u32 and usize type.
#[derive(Debug, Clone, Copy)]
pub struct U32AndUsize {
    u32: NonZeroU32,
    usize: NonZeroUsize,
}

impl fmt::Display for U32AndUsize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.u32)
    }
}

impl TryFrom<NonZeroU32> for U32AndUsize {
    type Error = TryFromIntError;
    fn try_from(value: NonZeroU32) -> Result<Self, Self::Error> {
        Ok(Self {
            u32: value,
            usize: value.try_into()?,
        })
    }
}

impl TryFrom<u32> for U32AndUsize {
    type Error = TryFromIntError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let nzvalue: NonZeroU32 = value.try_into()?;
        Ok(Self {
            u32: nzvalue,
            usize: nzvalue.try_into()?,
        })
    }
}

impl From<U32AndUsize> for usize {
    fn from(value: U32AndUsize) -> Self {
        value.usize.get()
    }
}

impl From<U32AndUsize> for NonZeroUsize {
    fn from(value: U32AndUsize) -> Self {
        value.usize
    }
}

impl From<U32AndUsize> for u32 {
    fn from(value: U32AndUsize) -> Self {
        value.u32.get()
    }
}

impl From<U32AndUsize> for NonZeroU32 {
    fn from(value: U32AndUsize) -> Self {
        value.u32
    }
}

impl From<U32AndUsize> for u64 {
    fn from(value: U32AndUsize) -> Self {
        value.u32.get().into()
    }
}

impl From<U32AndUsize> for f64 {
    fn from(value: U32AndUsize) -> Self {
        value.u32.get().into()
    }
}
