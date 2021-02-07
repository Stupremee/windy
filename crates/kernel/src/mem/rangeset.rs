//! A `RangeSet` which contains non-overlapping sets of
//! `u64` inclusive ranges. The `RangeSet` can be used to insert or remove
//! ranges of `u64`s and thus is very useful for physical memory management.

use core::{cmp, fmt, slice};

/// The number of ranges inside a fixed-size [`RangeSet`].
pub const RANGE_COUNT: usize = 32;

/// Any error that can occurr while operating on a [`RangeSet`].
#[derive(Clone, Debug)]
pub enum Error {
    /// The range was invalid, meaning that `start > end`.
    InvalidRange,
    /// A given index was out of bounds.
    OutOfBounds,
}

/// An inclusive range that implements [`Copy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    /// The start for this range.
    pub start: usize,
    /// The end for this range.
    pub end: usize,
}

impl Range {
    /// Create a new `Range` goes from `start..=end`.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// A fixed-size set of inclusive [ranges](Range).
///
/// To effectively use a [`RangeSet`], [push](RangeSet::push) all requested
/// memory regions into this set, and [remove](RangeSet::remove) all ranges
/// that should not be part of the allocator.
#[derive(Clone)]
pub struct RangeSet {
    /// The fixed array of ranges.
    ranges: [Range; RANGE_COUNT],

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

    /// Get the range at the given index if there's one present.
    pub fn get(&mut self, idx: usize) -> Option<Range> {
        if idx >= self.idx {
            None
        } else {
            Some(self.ranges[idx])
        }
    }

    /// Remove the range at the given index.
    pub fn remove(&mut self, idx: usize) -> Result<(), Error> {
        if idx >= self.idx {
            return Err(Error::OutOfBounds);
        }

        // we delete a range, by moving it out of bounds using `rotate_left`.
        //
        // Imagine this range set before the removal:
        //
        //           +-----------+
        //           |           v
        // +---+---+---+---+---+---+---+---+
        // | a | b | c | d | e |   |   |   |
        // +---+---+---+---+---+---+---+---+
        //           ^           ^
        //           |           |
        //          idx       self.idx
        //
        // This is how it looks like after the rotate left operation:
        //
        // +---+---+---+---+---+---+---+---+
        // | a | b | d | e |   |   |   | c |
        // +---+---+---+---+---+---+---+---+
        //                   ^
        //                   |
        //                self.idx
        //
        // So we basically "removing" a range by moving it out of bounds
        // so it can be overwritten
        self.ranges[idx..].rotate_left(1);
        self.idx -= 1;

        Ok(())
    }

    pub fn remove_range(&mut self, range: Range) -> Result<(), Error> {
        if range.start > range.end {
            return Err(Error::InvalidRange);
        }
        self.remove_inner(range);

        Ok(())
    }

    /// Loop through all ranges of this set and trim them, or even remove them,
    /// if they fit into `range` or just overlap.
    fn remove_inner(&mut self, range: Range) {
        for idx in 0..self.idx {
            let other = self.ranges[idx];

            if !overlaps(range, other) {
                continue;
            }

            if contains(other, range) {
                self.remove(idx).unwrap();
                self.remove_inner(range);
                return;
            }

            if range.start <= other.start {
                // If we have the following situtation:
                //
                // [======`range`======]
                //                [=====`other`=====]
                //
                // We trim the start of `other` to the end of `range` so it looks like this:
                //
                // [======`range`======]
                //                      [==`other`==]
                self.ranges[idx].start = range.end.saturating_add(1);
            } else if range.end >= other.end {
                // If we have the following situtation:
                //
                // [====`other`====]
                //                [=====`range`=====]
                //
                // We trim the end of `other` to the start of `range` so it looks like this:
                //
                // [====`other`==]
                //                [=====`range`=====]
                self.ranges[idx].end = range.start.saturating_sub(1);
            } else {
                // If we have the following situtation:
                //
                // [=========`other`=========]
                //    [=====`range`=====]
                //
                // We split `other` into two separate ranges:
                //
                // [==] <-- `other`       [==] <-- `new_range`
                //     [=====`range`=====]
                let new_range = Range::new(range.start.saturating_add(1), other.end);

                // we don't use `insert` here because `insert` would try
                // to merge blocks which would be quite expensive.
                self.ranges[self.idx] = new_range;
                self.idx += 1;

                self.ranges[idx].end = range.start.saturating_sub(1);
                self.remove_inner(range);
            }
        }
    }

    /// Insert a new range into this rangeset.
    ///
    /// If the range overlaps with another range inside this set,
    /// both ranges will be collapsed into a single range.
    pub fn insert(&mut self, mut range: Range) -> Result<(), Error> {
        if range.start > range.end {
            return Err(Error::InvalidRange);
        }

        self.merge_blocks(&mut range);

        self.ranges[self.idx] = range;
        self.idx += 1;

        Ok(())
    }

    /// Loop through all ranges and merge all blocks that either touch
    /// or overlap.
    fn merge_blocks(&mut self, range: &mut Range) {
        for idx in 0..self.idx {
            let other = self.ranges[idx];

            let a = Range::new(range.start, range.end.saturating_add(1));
            let b = Range::new(other.start, other.end.saturating_add(1));
            if !overlaps(a, b) {
                continue;
            }

            range.start = cmp::min(range.start, other.start);
            range.end = cmp::max(range.end, other.end);

            self.remove(idx).unwrap();

            self.merge_blocks(range);
        }
    }

    /// Remove all ranges from this rangeset.
    pub fn clear(&mut self) {
        self.idx = 0;
    }

    /// Return a slice that contains all ranges.
    #[inline]
    pub fn as_slice(&self) -> &[Range] {
        &self.ranges[..self.idx]
    }

    /// Return a mutable slice that contains all ranges.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [Range] {
        &mut self.ranges[..self.idx]
    }

    /// Return an iterator over all ranges of this set.
    pub fn iter(&self) -> slice::Iter<'_, Range> {
        self.as_slice().iter()
    }

    /// Return an immutable iterator over all ranges of this set.
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, Range> {
        self.as_mut_slice().iter_mut()
    }

    /// Return the number of ranges inside this rangeset.
    pub fn len(&self) -> usize {
        self.idx
    }

    /// Check if this range set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl fmt::Debug for RangeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RangeSet")
            .field("ranges", &self.as_slice())
            .field("idx", &self.idx)
            .finish()
    }
}

/// Check if two ranges overlap.
fn overlaps(a: Range, b: Range) -> bool {
    a.start <= b.end && b.start <= a.end
}

/// Check if the whole range of `a` fits inside the
/// range `b`.
fn contains(a: Range, b: Range) -> bool {
    a.start >= b.start && a.end <= b.end
}
