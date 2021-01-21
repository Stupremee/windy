use core::ops::{Bound, Range, RangeBounds};

/// A generic trait which provides methods for extracting and setting specific bits or ranges of
/// bits.
pub trait BitField {
    /// The number of bits this bit field has.
    const BIT_LENGTH: usize;

    /// Gets the bit at the given position.
    fn get_bit(&self, bit: usize) -> bool;

    /// Get multiple bits that are inside the given range.
    fn get_bits<T: RangeBounds<usize>>(&self, range: T) -> Self;

    /// Set the bit to the given value, either `1` or `0`.
    fn set_bit(&mut self, bit: usize, value: bool);

    /// Set the range of bits to the given bitfield.
    fn set_bits<T: RangeBounds<usize>>(&mut self, range: T, value: Self);
}

macro_rules! num_impl {
    ($($t:ty)*) => ($(
        impl BitField for $t {
            const BIT_LENGTH: usize = core::mem::size_of::<Self>() * 8;

            #[inline]
            fn get_bit(&self, bit: usize) -> bool {
                assert!(bit < Self::BIT_LENGTH);
                (*self & (1 << bit)) != 0
            }

            #[inline]
            fn get_bits<T: RangeBounds<usize>>(&self, range: T) -> Self {
                let range = normalize_range(&range, Self::BIT_LENGTH);

                assert!(range.start < Self::BIT_LENGTH);
                assert!(range.end <= Self::BIT_LENGTH);
                assert!(range.start < range.end);

                let bits = *self << (Self::BIT_LENGTH - range.end) >> (Self::BIT_LENGTH - range.end);
                bits >> range.start
            }

            #[inline]
            fn set_bit(&mut self, bit: usize, value: bool) {
                assert!(bit < Self::BIT_LENGTH);

                if value {
                    *self |= 1 << bit;
                } else {
                    *self &= !(1 << bit);
                }
            }

            #[inline]
            fn set_bits<T: RangeBounds<usize>>(&mut self, range: T, value: Self) {
                let range = normalize_range(&range, Self::BIT_LENGTH);

                assert!(range.start < Self::BIT_LENGTH);
                assert!(range.end <= Self::BIT_LENGTH);
                assert!(range.start < range.end);
                assert!(value << (Self::BIT_LENGTH - (range.end - range.start)) >>
                        (Self::BIT_LENGTH - (range.end - range.start)) == value,
                        "value does not fit into bit range");

                let bitmask: Self = !(!0 << (Self::BIT_LENGTH - range.end) >>
                                    (Self::BIT_LENGTH - range.end) >>
                                    range.start << range.start);

                *self = (*self & bitmask) | (value << range.start);
            }
        }
    )*)
}

num_impl! { u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize }

fn normalize_range<T: RangeBounds<usize>>(generic_rage: &T, bit_length: usize) -> Range<usize> {
    let start = match generic_rage.start_bound() {
        Bound::Excluded(&value) => value + 1,
        Bound::Included(&value) => value,
        Bound::Unbounded => 0,
    };
    let end = match generic_rage.end_bound() {
        Bound::Excluded(&value) => value,
        Bound::Included(&value) => value + 1,
        Bound::Unbounded => bit_length,
    };

    start..end
}
