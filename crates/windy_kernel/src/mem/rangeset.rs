//! A `RangeSet` which contains non-overlapping sets of
//! `u64` inclusive ranges. The `RangeSet` can be used to insert or remove
//! ranges of `u64`s and thus is very useful for physical memory management.

/// The number of ranges inside a fixed-size [`RangeSet`].
pub const RANGE_COUNT: usize = 32;

/// Any error that can occurr while operating on a [`RangeSet`].
#[derive(Clone)]
pub enum Error {
    /// The range was invalid, meaning that `start > end`.
    InvalidRange,
    /// A given index was out of bounds.
    OutOfBounds,
}

/// An inclusive range that implements [`Copy`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Range {
    /// The start for this range.
    pub start: usize,
    /// The end for this range.
    pub end: usize,
}

/// A fixed-size set of inclusive [ranges](Range).
///
/// To effectively use a [`RangeSet`], [push](RangeSet::push) all requested
/// memory regions into this set, and [remove](RangeSet::remove) all ranges
/// that should not be part of the allocator.
#[derive(Clone)]
pub struct RangeSet {
    /// The fixed array of ranges.
    ranges: [Range; 32],

    /// The current index inside the ranges array.
    idx: usize,
}

impl RangeSet {
    /// Create a new empty rangeset.
    pub const fn new() -> Self {
        Self {
            ranges: [Range { start: 0, end: 0 }; RANGE_COUNT],
            idx: 0,
        }
    }

    /// Remove the range at the given index.
    pub fn remove(&mut self, idx: usize) -> Result<(), Error> {
        if idx >= self.idx {
            return Err(Error::OutOfBounds);
        }

        todo!()
    }

    /// Insert a new range into this rangeset.
    ///
    /// If the range overlaps with another range inside this set,
    /// both ranges will be collapsed into a single range.
    pub fn insert(&mut self, range: Range) -> Result<(), Error> {
        if range.start > range.end {
            return Err(Error::InvalidRange);
        }

        Ok(())
    }
}
