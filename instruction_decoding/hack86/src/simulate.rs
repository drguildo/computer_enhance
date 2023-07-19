use crate::decode::{self, Instruction};

pub struct Registers {
    ax: u16,
    cx: u16,
    dx: u16,
    bx: u16,
    sp: u16,
    bp: u16,
    si: u16,
    di: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            ax: 0,
            bx: 0,
            cx: 0,
            dx: 0,
            bp: 0,
            sp: 0,
            di: 0,
            si: 0,
        }
    }

    pub fn simulate(&mut self, instruction: &Instruction) {
        match &instruction.instruction_category {
            decode::InstructionCategory::RegisterMemoryAndRegister(mnemonic, src, dest) => {}
            decode::InstructionCategory::ImmediateToRegister(mnemonic, immediate, register) => {
                match mnemonic {
                    decode::Mnemonic::ADD => todo!(),
                    decode::Mnemonic::CMP => todo!(),
                    decode::Mnemonic::JA => todo!(),
                    decode::Mnemonic::JC => todo!(),
                    decode::Mnemonic::JCXZ => todo!(),
                    decode::Mnemonic::JG => todo!(),
                    decode::Mnemonic::JL => todo!(),
                    decode::Mnemonic::JNA => todo!(),
                    decode::Mnemonic::JNC => todo!(),
                    decode::Mnemonic::JNG => todo!(),
                    decode::Mnemonic::JNL => todo!(),
                    decode::Mnemonic::JNO => todo!(),
                    decode::Mnemonic::JNS => todo!(),
                    decode::Mnemonic::JNZ => todo!(),
                    decode::Mnemonic::JO => todo!(),
                    decode::Mnemonic::JPE => todo!(),
                    decode::Mnemonic::JPO => todo!(),
                    decode::Mnemonic::JS => todo!(),
                    decode::Mnemonic::JZ => todo!(),
                    decode::Mnemonic::LOOP => todo!(),
                    decode::Mnemonic::LOOPE => todo!(),
                    decode::Mnemonic::LOOPNE => todo!(),
                    decode::Mnemonic::MOV => match register {
                        decode::RegisterName::AL => todo!(),
                        decode::RegisterName::BL => todo!(),
                        decode::RegisterName::CL => todo!(),
                        decode::RegisterName::DL => todo!(),
                        decode::RegisterName::AH => todo!(),
                        decode::RegisterName::BH => todo!(),
                        decode::RegisterName::CH => todo!(),
                        decode::RegisterName::DH => todo!(),
                        decode::RegisterName::AX => self.ax = *immediate,
                        decode::RegisterName::BX => self.bx = *immediate,
                        decode::RegisterName::CX => self.cx = *immediate,
                        decode::RegisterName::DX => self.dx = *immediate,
                        decode::RegisterName::BP => self.bp = *immediate,
                        decode::RegisterName::SP => self.sp = *immediate,
                        decode::RegisterName::DI => self.di = *immediate,
                        decode::RegisterName::SI => self.si = *immediate,
                    },
                    decode::Mnemonic::SUB => todo!(),
                }
            }
            decode::InstructionCategory::ImmediateToRegisterMemory(
                mnemonic,
                immediate,
                dest,
                word_operation,
            ) => {}
            decode::InstructionCategory::ImmediateToAccumulator(mnemonic, immediate, dest) => {}
            decode::InstructionCategory::Jump(mnemonic, increment) => {}
        };
    }
}

impl std::fmt::Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push_str(&format!("ax: {:#06x} ({})\n", self.ax, self.ax));
        s.push_str(&format!("bx: {:#06x} ({})\n", self.bx, self.bx));
        s.push_str(&format!("cx: {:#06x} ({})\n", self.cx, self.cx));
        s.push_str(&format!("dx: {:#06x} ({})\n", self.dx, self.dx));
        s.push_str(&format!("sp: {:#06x} ({})\n", self.sp, self.sp));
        s.push_str(&format!("bp: {:#06x} ({})\n", self.bp, self.bp));
        s.push_str(&format!("si: {:#06x} ({})\n", self.si, self.si));
        s.push_str(&format!("di: {:#06x} ({})\n", self.di, self.di));
        write!(f, "{}", s)
    }
}
