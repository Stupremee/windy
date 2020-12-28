use crate::{Error, SbiResult};

/// A Hart mask is a list of harts that was given to a SBI
/// call using a bit-vector.
pub(crate) struct HartMask {
    mask: u64,
    base: u64,
}

impl HartMask {
    /// Create a  new [`HartMask`] from a hart mask and a hart base,
    /// which were given to a SBI.
    pub(crate) fn new(mask: u64, base: u64) -> Self {
        Self { mask, base }
    }

    /// Executes an action for every hart ID that is specified
    /// using the mask and base.
    ///
    /// The argument to the action is the correspondig Hart Id.
    pub(crate) fn for_each<A: FnMut(usize)>(self, mut action: A) -> SbiResult<()> {
        let platform = crate::platform::global().lock();
        let guard = platform.as_ref().ok_or(Error::Failed)?;

        // TODO: Hart State Management

        // `hart_count` is the given hart count, minuus
        let hart_count = (guard.hart_count as u64)
            .checked_sub(self.base)
            .ok_or(Error::InvalidParam)?;
        // `&` the real mask with the mask of all available harts,
        // so we only execute `action` for every available hart.
        let mut mask = self.mask & (1 << hart_count - 1);

        // loop through
        let mut idx = 0u64;
        while mask > 0 {
            if mask & 0x01 != 0 {
                let id = self.base + idx;
                action(id as usize);
            }
            mask >>= 1;
            idx += 1;
        }

        todo!()
    }
}
