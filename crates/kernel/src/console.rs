//! Implementation for accessing stdout/stdin.

use crate::drivers;

/// All different kinds of devices that can be used as a console.
pub enum ConsoleDevice {
    NS16550(drivers::ns16550a::Device),
}

impl ConsoleDevice {
    /// Retrieve a console device from the `/chosen` node that is
    /// inside the devicetree.
    ///
    /// # Safety
    ///
    /// The given `node` must come from the devicetree to be a vaild node.
    pub unsafe fn from_chosen(node: &devicetree::node::ChosenNode<'_>) -> Option<Self> {
        let stdout = node.stdout()?;
        let mut compatible = stdout.prop("compatible")?.as_strings();

        if compatible.any(|name| drivers::ns16550a::COMPATIBLE.contains(&name)) {
            let addr = stdout.regions().next()?.start();
            Some(Self::NS16550(drivers::ns16550a::Device::new(addr)))
        } else {
            None
        }
    }
}
