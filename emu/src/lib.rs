#[macro_use]
#[cfg(test)]
extern crate itertools;
extern crate r68k_common;
#[cfg(test)]
extern crate r68k_tools;

pub mod cpu;
#[macro_use]
pub mod ram;
pub mod interrupts;
pub mod musashi;


#[cfg(test)]
mod tests {
    use cpu::TestCore;
    use r68k_tools::memory::MemoryVec;
    use r68k_tools::PC;
    use r68k_tools::Exception;
    use cpu::ops::handlers::InstructionSetGenerator;
    use r68k_tools::disassembler::Disassembler;

    #[test]
    // #[ignore]
    fn roundtrips() {
        let mut over = 0;
        let mut under = 0;
        let mut wrong = 0;
        let gen = InstructionSetGenerator::<TestCore>::new();
        let optable: Vec<&str> = gen.generate_with("???", |ref op| op.name);
        let d = Disassembler::new();
        for opcode in 0x0000..0xffff {
            let op = optable[opcode];
            let parts:Vec<&str> = op.split('_').collect();
            let mnemonic = parts[0];
            let pc = PC(0);
            let extension_word_mask = 0b1111_1000_1111_1111;
            // bits 8-10 should always be zero in the ea extension word
            // as we don't know which word will be seen as the ea extension word
            // (as opposed to immediate operand values) just make sure these aren't set.
            let dasm_mem = &mut MemoryVec::new16(pc, vec![opcode as u16, 0x001f, 0x00a4, 0x1234 & extension_word_mask, 0x5678 & extension_word_mask]);
            // println!("PREDASM {:04x}", opcode);
            match d.disassemble(pc, dasm_mem) {
                Err(Exception::IllegalInstruction(_opcode, _)) => if op != "???" && op != "unimplemented_1111" && op != "unimplemented_1010" && op != "illegal" {
                    under += 1;
                    println!("{:04x}: {} disasm under", opcode, op);
                }
                , //println!("{:04x}:\t\tover", opcode),
                Ok((_, dis_inst)) => if op == "???" || op == "unimplemented_1111" || op == "unimplemented_1010" || op == "illegal" {
                    over += 1;
                    println!("{:04x}: {} disasm over, {}", opcode, op, dis_inst);
                } else if dis_inst.mnemonic.to_lowercase() != mnemonic && mnemonic != "real" { // ILLEGAL == real_illegal
                    wrong += 1;
                    println!("{:04x}: {} disasm different {}", opcode, op, dis_inst);
                },
            }
        };
        println!("{}  opcodes over, {} under, {} wrong", over, under, wrong);
    }
}

/// A macro similar to `vec![$elem; $size]` which returns a boxed array.
///
/// ```rustc
///     let _: Box<[u8; 1024]> = box_array![0; 1024];
/// ```
#[macro_export]
macro_rules! box_array {
    ($val:expr ; $len:expr) => {{
        // Use a generic function so that the pointer cast remains type-safe
        fn vec_to_boxed_array<T>(vec: Vec<T>) -> Box<[T; $len]> {
            let boxed_slice = vec.into_boxed_slice();

            let ptr = ::std::boxed::Box::into_raw(boxed_slice) as *mut [T; $len];

            unsafe { Box::from_raw(ptr) }
        }

        vec_to_boxed_array(vec![$val; $len])
    }};
}