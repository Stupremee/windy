//! Wrappers around all primitive number types, but they
//! are layed out as big endian in memory.

/// Transparent wrapper around a primitive number, but
/// can be transmuted from a big endian memory representation.
#[derive(Debug)]
#[repr(transparent)]
pub struct BigEndian<Num>(Num);

impl<Num: PrimInt> BigEndian<Num> {
    /// Converts the inner number to big endian and returns it.
    pub fn get(&self) -> Num {
        self.0.to_be()
    }
}

macro_rules! impl_trait {
    (sealed: $($t:ty),*) => {
        $(
            impl Sealed for $t {}
        )*
    };

    ($($t:ty),*) => {
        $(
            impl PrimInt for $t {
                fn to_be(self) -> Self {
                    <$t>::to_be(self)
                }
            }
        )*
    };
}

mod sealed {
    pub trait Sealed {}

    impl_trait!(sealed: u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);
}

/// Represents any primitive number.
pub trait PrimInt: Sized + Copy {
    /// Converts `self` to big endian.
    fn to_be(self) -> Self;
}

impl_trait!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);
