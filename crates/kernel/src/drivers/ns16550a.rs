//! Driver for the `ns16550a` UART chip.

use self::registers::*;

mod registers {
    use rumio::{define_mmio_register, define_mmio_struct, mmio::Lit};

    define_mmio_register! {
        /// The Line-Status-Register
        LSR: u8 {
            /// Indicate if there's data to read
            r DATA_READY: 0,
            /// Check if the transmitter lane is empty
            r TRANSMITTER_EMPTY: 6,
        }
    }

    define_mmio_register! {
        /// The FIFO-Control-Register
        FCR: u8 {
            /// Enable FIFO
            rw FIFO_ENABLE: 0,
        }
    }

    define_mmio_register! {
        /// The Line-Control-Register
        LCR: u8 {
            /// Enable FIFO
            rw WORD_LEN: 0..1 = enum WordLen [
                Five = 0b00,
                Six = 0b01,
                Seven = 0b10,
                Eight = 0b11,
            ],

            /// Divisor-Latch access bit.
            rw DLAB: 7,
        }
    }

    define_mmio_struct! {
        /// The raw MMIO block for controlling the ns16550a chip.
        pub struct Registers {
            (0x00 => DATA: Lit<u8>),
            (0x02 => FCR: FCR),
            (0x03 => LCR: LCR),
            (0x05 => LSR: LSR),
        }
    }
}

/// Driver for the `ns16550a` chip.
pub struct Device {
    regs: Registers,
}

impl Device {
    /// Create a new `ns16550a` device at the given base address.
    ///
    /// # Safety
    ///
    /// The `addr` must be a valid `ns16550a` MMIO device.
    pub const unsafe fn new(addr: usize) -> Self {
        Self {
            regs: Registers::new(addr),
        }
    }

    /// Prepares everything for this Uart driver to work properly.
    pub fn init(&mut self) {
        // enable FIFO mode so the messages are sent in order
        self.regs.FCR().FIFO_ENABLE().set(true);

        // set the word length to 8 bits
        self.regs.LCR().WORD_LEN().set(WordLen::Eight);

        // TODO:
        // enable data available interrupt
        // self.regs.ier().modify(IER::DATA_READY::SET);
    }

    /// Set the baud rate that is used for transmitting data.
    ///
    /// `rate` is the new baud rate, and `clock` must be the frequency of
    /// the clock that is driving this uart device.
    pub fn set_baud_rate(&mut self, rate: u32, clock: u32) {
        // calculate the divisor using the formular and then
        // split into lower and higher bytes.
        let divisor = (clock / (rate * 16)) as u16;
        let low = (divisor & 0xFF) as u8;
        let high = (divisor >> 8) as u8;

        // enable DLAB so we can write the divisor
        self.regs.LCR().DLAB().set(true);

        // write the two divisor bytes
        self.regs.DATA().write(low);
        self.regs.DATA().write(high);

        // disable DLAB so we can transmit/receive data again
        self.regs.LCR().DLAB().set(false);
    }

    /// Tries to read incoming data, but will return `None`
    /// if there's no data available.
    pub fn try_read(&mut self) -> Option<u8> {
        self.data_ready().then(|| unsafe { self.read_data() })
    }

    /// Spins this hart until there's data available
    pub fn read(&mut self) -> u8 {
        while !self.data_ready() {}
        unsafe { self.read_data() }
    }

    /// Tries to send data but will fail if the transmitter is not empty.
    ///
    /// Returns `Some(x)` with the given value if the transmitter was not empty
    /// and the data couldn't be send.
    pub fn try_write(&mut self, x: u8) -> Option<u8> {
        if self.transmitter_empty() {
            unsafe { self.write_data(x) };
            None
        } else {
            Some(x)
        }
    }

    /// Spins this hart until the data can be send.
    pub fn write(&mut self, x: u8) {
        while !self.transmitter_empty() {}
        unsafe { self.write_data(x) };
    }

    /// Reads data from the receiver holding register.
    ///
    /// # Safety
    ///
    /// Must only be called if there's data available.
    pub unsafe fn read_data(&mut self) -> u8 {
        self.regs.DATA().read()
    }

    /// Writes the given byte into the transmitter holding register.
    ///
    /// # Safety
    ///
    /// Must only be called if the transmitter is ready and empty.
    pub unsafe fn write_data(&mut self, x: u8) {
        self.regs.DATA().write(x);
    }

    fn transmitter_empty(&self) -> bool {
        self.regs.LSR().read(TRANSMITTER_EMPTY::FIELD) != 0
    }

    fn data_ready(&self) -> bool {
        self.regs.LSR().read(DATA_READY::FIELD) != 0
    }
}
