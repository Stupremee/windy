//! Driver implementation for the ns16550a Uart circuit.

use core::fmt;
use register::{mmio::ReadWrite, register_bitfields, register_structs};

register_bitfields! {
    u8,

    /// Interrupt Enable Register
    pub IER [
        DATA_READY 0,
        THR_EMPTY 1,
        RECV_LINE 2
    ],

    /// FIFO Control Register
    pub FCR [
        FIFO_ENABLE 0
    ],

    /// Line Control Register
    pub LCR [
        /// Specify the length of a single word in bits.
        WORD_LENGTH OFFSET(0) NUMBITS(2) [
            FIVE = 0b00,
            SIX = 0b01,
            SEVEN = 0b10,
            EIGHT = 0b11
        ],

        DLAB OFFSET(7) NUMBITS(1) []
    ],

    /// The line-status register
    pub LSR [
        DATA_READY OFFSET(0) NUMBITS(1) [],
        TRANSMITTER_EMPTY OFFSET(6) NUMBITS(1) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub Registers {
        (0x00 => pub DATA: ReadWrite<u8>),
        (0x01 => pub IER: ReadWrite<u8, IER::Register>),
        (0x02 => pub FCR: ReadWrite<u8, FCR::Register>),
        (0x03 => pub LCR: ReadWrite<u8, LCR::Register>),
        (0x04 => _reserved0: ReadWrite<u8>),
        (0x05 => pub LSR: ReadWrite<u8, LSR::Register>),
        (0x06 => @END),
    }
}

/// Uart driver for the ns16550a chip.
pub struct Uart {
    base: *mut Registers,
}

impl Uart {
    /// Create a new Uart device that is mapped at the given address.
    pub const fn new(base: *mut u8) -> Self {
        Self {
            base: base as *mut _,
        }
    }

    /// Prepares everything for this Uart driver to work properly.
    pub fn init(&mut self) {
        let registers = unsafe { &*self.base };

        // enable FIFO mode so the messages are sent in order
        registers.FCR.modify(FCR::FIFO_ENABLE::SET);

        // set the word length to 8 bits
        registers.LCR.modify(LCR::WORD_LENGTH::EIGHT);

        // enable data available interrupt
        registers.IER.modify(IER::DATA_READY::SET);
    }

    /// Set the baud rate to the given value.
    ///
    /// It also requires the clock frequency to calculate the divisor.
    pub fn set_baud_rate(&mut self, freq: u32, rate: u32) {
        let registers = unsafe { &*self.base };

        // calculate the divisor using the formular and then
        // split into lower and higher bytes.
        let divisor = (freq / (rate * 16)) as u16;
        let low = (divisor & 0xFF) as u8;
        let high = (divisor >> 8) as u8;

        // enable DLAB so we can write the divisor
        registers.LCR.modify(LCR::DLAB::SET);

        // write the two divisor bytes
        registers.DATA.set(low);
        registers.IER.set(high);

        // disable DLAB so we can transmit/receive data again
        registers.LCR.modify(LCR::DLAB::CLEAR);
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
        let registers = &*self.base;
        registers.DATA.get()
    }

    /// Writes the given byte into the transmitter holding register.
    ///
    /// # Safety
    ///
    /// Must only be called if the transmitter is ready and empty.
    pub unsafe fn write_data(&mut self, x: u8) {
        let registers = &*self.base;
        registers.DATA.set(x);
    }

    fn transmitter_empty(&self) -> bool {
        let registers = unsafe { &*self.base };
        registers.LSR.read(LSR::TRANSMITTER_EMPTY) != 0
    }

    fn data_ready(&self) -> bool {
        let registers = unsafe { &*self.base };
        registers.LSR.read(LSR::DATA_READY) != 0
    }
}

unsafe impl Send for Uart {}
unsafe impl Sync for Uart {}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for x in s.bytes() {
            self.write(x);
        }
        Ok(())
    }
}
