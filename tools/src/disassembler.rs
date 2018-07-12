use operand::Operand;
use memory::Memory;
use constants::*;
use super::{Result, Size, Exception,OpcodeInstance,generate};

fn decode_destination_ea(opcode: u16, size: Size, pc: u32, mem: &Memory) -> Operand {
    let mode = ((opcode >> 6) & 0b111) as u8;
    let reg_y = ((opcode >> 9) & 0b111) as u8;
    effective_address(size, pc, mem, mode, reg_y)
}

fn decode_ea(opcode: u16, size: Size, pc: u32, mem: &Memory) -> Operand {
    let mode = ((opcode >> 3) & 0b111) as u8;
    let reg_y = (opcode & 0b111) as u8;
    effective_address(size, pc, mem, mode, reg_y)
}

fn effective_address(size: Size, pc: u32, mem: &Memory, mode: u8, reg_y: u8) -> Operand {
    match mode {
        0b000 => Operand::DataRegisterDirect(reg_y),
        0b001 => Operand::AddressRegisterDirect(reg_y),
        0b010 => Operand::AddressRegisterIndirect(reg_y),
        0b011 => Operand::AddressRegisterIndirectWithPostincrement(reg_y),
        0b100 => Operand::AddressRegisterIndirectWithPredecrement(reg_y),
        0b101 => Operand::AddressRegisterIndirectWithDisplacement(reg_y, mem.read_word(pc + 2) as i16),
        0b110 => {
            let (indexinfo, displacement) = decode_extension_word(mem.read_word(pc + 2));
            Operand::AddressRegisterIndirectWithIndex(reg_y, indexinfo, displacement)
        },
        0b111 => match reg_y {
            0b010 => Operand::PcWithDisplacement(mem.read_word(pc + 2) as i16),
            0b011 => {
                let (indexinfo, displacement) = decode_extension_word(mem.read_word(pc + 2));
                Operand::PcWithIndex(indexinfo, displacement)
            },
            0b000 => Operand::AbsoluteWord(mem.read_word(pc + 2)),
            0b001 => Operand::AbsoluteLong((mem.read_word(pc + 2) as u32) << 16 | mem.read_word(pc + 4) as u32),
            0b100 =>
                match size {
                    Size::Byte => Operand::Immediate(size, (mem.read_word(pc + 2) & 0xFF) as u32),
                    Size::Word => Operand::Immediate(size, mem.read_word(pc + 2) as u32),
                    Size::Long => Operand::Immediate(size, (mem.read_word(pc + 2) as u32) << 16 | mem.read_word(pc + 4) as u32),
                    Size::Unsized => panic!("unsized Immediate"),
                },
            _ => panic!("Unknown addressing mode {:03b} reg {:03b}", mode, reg_y),
        },
        _ => panic!("Unknown addressing mode {:03b} reg {:03b}", mode, reg_y),
    }
}

fn decode_extension_word(extension: u16) -> (u8, i8) {
    // top four bits = (D/A RRR) matches our register array layout
    let xreg_ndx_size = (extension>>12) as u8;
    let displacement = extension as i8;
    (xreg_ndx_size, displacement)
}
#[allow(unused_variables)]
fn decode_dx(opcode: u16, pc: u32, mem: &Memory) -> Operand {
    Operand::DataRegisterDirect(((opcode >> 9) & 7) as u8)
}
#[allow(unused_variables)]
fn decode_ax(opcode: u16, pc: u32, mem: &Memory) -> Operand {
    Operand::AddressRegisterDirect(((opcode >> 9) & 7) as u8)
}
fn decode_imm(size: Size, pc: u32, mem: &Memory) -> Operand {
    match size {
        Size::Byte => Operand::Immediate(size, (mem.read_word(pc+2) & 0xFF) as u32),
        Size::Word => Operand::Immediate(size, mem.read_word(pc+2) as u32),
        Size::Long => Operand::Immediate(size, (mem.read_word(pc+2) as u32) << 16 | mem.read_word(pc+4) as u32),
        Size::Unsized => panic!("unsized Immediate"),
    }
}
pub fn decode_ea_sr(opcode: u16, size: Size, pc: u32, mem: &Memory) -> Vec<Operand> {
    vec![decode_ea(opcode, size, pc, mem), Operand::StatusRegister(Size::Word)]
}
pub fn decode_ea_ccr(opcode: u16, size: Size, pc: u32, mem: &Memory) -> Vec<Operand> {
    vec![decode_ea(opcode, size, pc, mem), Operand::StatusRegister(Size::Byte)]
}
pub fn decode_ea_dx(opcode: u16, size: Size, pc: u32, mem: &Memory) -> Vec<Operand> {
    vec![decode_ea(opcode, size, pc, mem), decode_dx(opcode, pc, mem)]
}
pub fn decode_ea_ea(opcode: u16, size: Size, pc: u32, mem: &Memory) -> Vec<Operand> {
    let src = decode_ea(opcode, size, pc, mem);
    let dst = decode_destination_ea(opcode, size, pc + src.extension_words() * 2, mem);
    vec![src, dst]
}
pub fn decode_ea_ax(opcode: u16, size: Size, pc: u32, mem: &Memory) -> Vec<Operand> {
    vec![decode_ea(opcode, size, pc, mem), decode_ax(opcode, pc, mem)]
}
pub fn decode_dx_ea(opcode: u16, size: Size, pc: u32, mem: &Memory) -> Vec<Operand> {
    vec![decode_dx(opcode, pc, mem), decode_ea(opcode, size, pc, mem)]
}
pub fn decode_imm_ea(opcode: u16, size: Size, pc: u32, mem: &Memory) -> Vec<Operand> {
    let imm = decode_imm(size, pc, mem);
    vec![imm, decode_ea(opcode, size, pc + imm.extension_words()*2, mem)]
}

pub fn decode_branch(opcode: u16, _size: Size, pc: u32, mem: &Memory) -> Vec<Operand> {
    let disp8 = opcode & 0xFF;
    let displacement = if disp8 > 0 && disp8 < 0xff {
        Operand::Displacement(Size::Byte, disp8 as u32)
    } else if disp8 == 00 {
        Operand::Displacement(Size::Word, mem.read_word(pc+2) as u32)
    } else if disp8 == 0xff {
        Operand::Displacement(Size::Long, (mem.read_word(pc+2) as u32) << 16 | mem.read_word(pc+4) as u32)
    } else {
        unreachable!()
    };
    vec![displacement]
}

/* Check if opcode is using a valid ea mode */
pub fn valid_ea(opcode: u16, ea_mask: u16) -> bool
{
    if ea_mask == 0 {
        true
    } else {
        // ea is the lower six bits of the opcode
        match opcode & 0x3f {
            0x00 ... 0x07 => (ea_mask & EA_DATA_REGISTER_DIRECT) != 0,
            0x08 ... 0x0f => (ea_mask & EA_ADDRESS_REGISTER_DIRECT) != 0,
            0x10 ... 0x17 => (ea_mask & EA_ADDRESS_REGISTER_INDIRECT) != 0,
            0x18 ... 0x1f => (ea_mask & EA_ARI_POSTINCREMENT) != 0,
            0x20 ... 0x27 => (ea_mask & EA_ARI_PREDECREMENT) != 0,
            0x28 ... 0x2f => (ea_mask & EA_ARI_DISPLACEMENT) != 0,
            0x30 ... 0x37 => (ea_mask & EA_ARI_INDEX) != 0,
            0x38 => (ea_mask & EA_ABSOLUTE_SHORT) != 0,
            0x39 => (ea_mask & EA_ABSOLUTE_LONG) != 0,
            0x3a => (ea_mask & EA_PC_DISPLACEMENT) != 0,
            0x3b => (ea_mask & EA_PC_INDEX) != 0,
            0x3c => (ea_mask & EA_IMMEDIATE) != 0,
            _ => false
        }
    }
}

fn get_dest_ea(opcode: u16) -> u16 {
    // normally ea are the 6 least significant bits structured as mmmrrr
    // but for move op codes, the destination ea is stored as rrrmmm000000
    // (where the lower six bits are the source ea)
    // we need to swap and shift that into place
    let mode = (opcode >> 3) & 0b111000;
    let reg_y = (opcode >> 9) & 0b111;
    mode | reg_y
}

pub fn ea_memory_alterable(opcode: u16) -> bool { valid_ea(opcode, EA_MEMORY_ALTERABLE) }
pub fn ea_all_except_an(opcode: u16) -> bool { valid_ea(opcode, EA_ALL_EXCEPT_AN) }
pub fn ea_all(opcode: u16) -> bool { valid_ea(opcode, EA_ALL) }
pub fn ea_data_alterable(opcode: u16) -> bool { valid_ea(opcode, EA_DATA_ALTERABLE) }
pub fn ea_all_to_data_alterable(opcode: u16) -> bool { valid_ea(opcode, EA_ALL) && valid_ea(get_dest_ea(opcode), EA_DATA_ALTERABLE) }
pub fn ea_data(opcode: u16) -> bool { valid_ea(opcode, EA_DATA) }
pub fn ea_control(opcode: u16) -> bool { valid_ea(opcode, EA_CONTROL) }
pub fn always(_opcode: u16) -> bool { true }
pub fn valid_byte_displacement(opcode: u16) -> bool {
    let disp8 = opcode & 0xff;
    disp8 > 0 && disp8 < 0xff
}
pub fn never(_opcode: u16) -> bool { false }

pub fn disassemble_first(mem: &Memory) -> OpcodeInstance {
    disassemble(0, mem).unwrap()
}

pub fn disassemble(pc: u32, mem: &Memory) -> Result<OpcodeInstance> {
    let optable = generate();
    let opcode = mem.read_word(pc);
    // println!("opcode read was {:04x}", opcode);
    for op in optable {
        // check for mask/opcode inconsistency
        assert!(op.mask & op.matching == op.matching, format!("mask/matching mismatch {:04x} & {:04x} for {}{}", op.mask, op.matching, op.mnemonic, op.size));
        if ((opcode as u32) & op.mask) == op.matching && (op.validator)(opcode) {
            let decoder = op.decoder;
            return Ok(OpcodeInstance {mnemonic: op.mnemonic, size: op.size, operands: decoder(opcode, op.size, pc, mem)});
        } else if ((opcode as u32) & op.mask) == op.matching {
            println!("{:04x}: match for {}{} without passing validator", opcode, op.mnemonic, op.size);
        }
    }
    Err(Exception::IllegalInstruction(opcode, pc))
}


#[cfg(test)]
mod tests {
    use operand::Operand;
    use memory::MemoryVec;
    use super::disassemble_first;
    use super::super::Size;

    #[test]
    fn decodes_add_8_er() {
        let mem = MemoryVec::new16(0, vec![0xd411]);
        let inst = disassemble_first(&mem);
        assert_eq!("ADD", inst.mnemonic);
        assert_eq!(Size::Byte, inst.size);
        assert_eq!(Operand::AddressRegisterIndirect(1), inst.operands[0]);
        assert_eq!(Operand::DataRegisterDirect(2), inst.operands[1]);
        assert_eq!("(A1)", format!("{}", inst.operands[0]));
        assert_eq!("D2", format!("{}", inst.operands[1]));
        assert_eq!("ADD.B\t(A1),D2", format!("{}", inst));
    }
    #[test]
    fn decodes_add_8_re() {
        let mem = MemoryVec::new16(0, vec![0xd511]);
        let inst = disassemble_first(&mem);

        assert_eq!("ADD", inst.mnemonic);
        assert_eq!(Size::Byte, inst.size);
        assert_eq!(Operand::DataRegisterDirect(2), inst.operands[0]);
        assert_eq!(Operand::AddressRegisterIndirect(1), inst.operands[1]);
        assert_eq!("D2", format!("{}", inst.operands[0]));
        assert_eq!("(A1)", format!("{}", inst.operands[1]));
        assert_eq!("ADD.B\tD2,(A1)", format!("{}", inst));
    }

    #[test]
    fn two_word_decode_imm_ea() {
        // ADDI #$12,$34(A0) is 0x0668 0x0012 0x0034
        let opcode = 0x0668;
        let dasm_mem = &mut MemoryVec::new16(0, vec![opcode, 0x0012, 0x0034]) ;
        let ops = super::decode_imm_ea(opcode, Size::Byte, 0, dasm_mem);
        assert_eq!(ops[0], Operand::Immediate(Size::Byte, 0x12));
        assert_eq!(ops[1], Operand::AddressRegisterIndirectWithDisplacement(0, 0x34));
    }
    #[test]
    fn three_word_decode_imm_ea_di() {
        // ADDI.L #$1F,$77(A6) is 0x06AE 0x0000 0x001F 0x0077
        let opcode = 0x06AE;
        let dasm_mem = &mut MemoryVec::new16(0, vec![opcode, 0x0000, 0x001F, 0x0077]);
        let ops = super::decode_imm_ea(opcode, Size::Long, 0, dasm_mem);
        assert_eq!(ops[0], Operand::Immediate(Size::Long, 0x1F));
        assert_eq!(ops[1], Operand::AddressRegisterIndirectWithDisplacement(6, 0x77));
    }
    #[test]
    fn three_word_decode_imm_ea_ix() {
        // ADDI.L #$1F00A4,52(A5,D2) is 0x06B5 0x001F 0x00A4 0x2034
        let opcode = 0x06B5;
        let dasm_mem = &mut MemoryVec::new16(0, vec![opcode, 0x001F, 0x00A4, 0x2034]);
        let ops = super::decode_imm_ea(opcode, Size::Long, 0, dasm_mem);
        assert_eq!(ops[0], Operand::Immediate(Size::Long, 0x1F00A4));
        assert_eq!(ops[1], Operand::AddressRegisterIndirectWithIndex(5, 2, 0x34));
    }
    #[test]
    fn four_word_decode_imm_ea_al() {
        // ADDI.L #$1F00A4,$12345678 is 0x06B9 0x001F 0x00A4 0x1234 0x5678
        let opcode = 0x06B9;
        let dasm_mem = &mut MemoryVec::new16(0, vec![opcode, 0x001F, 0x00A4, 0x1234, 0x5678]);
        let ops = super::decode_imm_ea(opcode, Size::Long, 0, dasm_mem);
        assert_eq!(ops[0], Operand::Immediate(Size::Long, 0x1F00A4));
        assert_eq!(ops[1], Operand::AbsoluteLong(0x12345678));
    }
}

