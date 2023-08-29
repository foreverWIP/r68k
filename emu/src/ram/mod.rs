pub mod loggingmem;
pub mod pagedmem;
pub use self::pagedmem::PagedMem;
// The m68k had a 24 bit external address bus with
// (2^24 bytes = ) 16 MB addressable space
pub const ADDRBUS_MASK: u32 = 0x00ff_ffff;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct AddressSpace(Mode, Segment);

impl AddressSpace {
    pub fn fc(self) -> u32 {
        match self {
            USER_DATA => 1,
            USER_PROGRAM => 2,
            SUPERVISOR_DATA => 5,
            SUPERVISOR_PROGRAM => 6,
        }
    }
    pub fn from_flags(s_flag: bool, fc_is_data: bool) -> Self {
        match (s_flag, fc_is_data) {
            (true, true) => SUPERVISOR_DATA,
            (true, false) => SUPERVISOR_PROGRAM,
            (false, true) => USER_DATA,
            (false, false) => USER_PROGRAM,
        }
    }
    pub fn from_musashi(value: u32) -> Self {
        match value & 0b111 {
            0b001 => USER_DATA,
            0b010 => USER_PROGRAM,
            0b101 => SUPERVISOR_DATA,
            0b110 => SUPERVISOR_PROGRAM,
            _ => panic!("unknown fc: {}", value),
        }
    }
}
use std::fmt;
impl fmt::Debug for AddressSpace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AddressSpace(mode, segment) => write!(f, "[{:?}/{:?}]", mode, segment),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
enum Segment {
    Program, Data
}
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
enum Mode {
    User, Supervisor
}

pub const SUPERVISOR_PROGRAM: AddressSpace = AddressSpace(Mode::Supervisor, Segment::Program);
pub const SUPERVISOR_DATA: AddressSpace = AddressSpace(Mode::Supervisor, Segment::Data);
pub const USER_PROGRAM: AddressSpace = AddressSpace(Mode::User, Segment::Program);
pub const USER_DATA: AddressSpace = AddressSpace(Mode::User, Segment::Data);

pub trait AddressBus {
    fn copy_from(&mut self, other: &Self);
    fn read_byte(&self, address_space: AddressSpace, address: u32) -> u32;
    fn read_word(&self, address_space: AddressSpace, address: u32) -> u32;
    fn read_long(&self, address_space: AddressSpace, address: u32) -> u32;
    fn write_byte(&mut self, address_space: AddressSpace, address: u32, value: u32);
    fn write_word(&mut self, address_space: AddressSpace, address: u32, value: u32);
    fn write_long(&mut self, address_space: AddressSpace, address: u32, value: u32);
}

