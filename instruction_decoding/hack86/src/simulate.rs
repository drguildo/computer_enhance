use crate::decode::{self, Instruction, RegisterName};

pub struct Register(RegisterName, u16);

impl Register {
    pub fn set(&mut self, new_value: u16) {
        println!("{}:{:#x}->{:#x}", self.0, self.1, new_value);
        self.1 = new_value;
    }
}

pub struct Registers {
    ax: Register,
    cx: Register,
    dx: Register,
    bx: Register,
    sp: Register,
    bp: Register,
    si: Register,
    di: Register,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            ax: Register(RegisterName::AX, 0),
            bx: Register(RegisterName::BX, 0),
            cx: Register(RegisterName::CX, 0),
            dx: Register(RegisterName::DX, 0),
            bp: Register(RegisterName::BP, 0),
            sp: Register(RegisterName::SP, 0),
            di: Register(RegisterName::DI, 0),
            si: Register(RegisterName::SI, 0),
        }
    }

    pub fn simulate(&mut self, instruction: &Instruction) {
        print!("{} ; ", instruction.instruction_category);
        match &instruction.instruction_category {
            decode::InstructionCategory::RegisterMemoryAndRegister(mnemonic, src, dest) => todo!(),
            decode::InstructionCategory::ImmediateToRegister(mnemonic, immediate, register) => {
                match mnemonic {
                    decode::Mnemonic::MOV => match register {
                        RegisterName::AL => todo!(),
                        RegisterName::BL => todo!(),
                        RegisterName::CL => todo!(),
                        RegisterName::DL => todo!(),
                        RegisterName::AH => todo!(),
                        RegisterName::BH => todo!(),
                        RegisterName::CH => todo!(),
                        RegisterName::DH => todo!(),
                        RegisterName::AX => self.ax.set(*immediate),
                        RegisterName::BX => self.bx.set(*immediate),
                        RegisterName::CX => self.cx.set(*immediate),
                        RegisterName::DX => self.dx.set(*immediate),
                        RegisterName::BP => self.bp.set(*immediate),
                        RegisterName::SP => self.sp.set(*immediate),
                        RegisterName::DI => self.di.set(*immediate),
                        RegisterName::SI => self.si.set(*immediate),
                    },
                    _ => todo!(),
                }
            }
            decode::InstructionCategory::ImmediateToRegisterMemory(
                mnemonic,
                immediate,
                dest,
                word_operation,
            ) => todo!(),
            decode::InstructionCategory::ImmediateToAccumulator(mnemonic, immediate, dest) => {
                todo!()
            }
            decode::InstructionCategory::Jump(mnemonic, increment) => todo!(),
        };
    }
}

impl std::fmt::Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push_str(&format!("ax: {:#06x} ({})\n", self.ax.1, self.ax.1));
        s.push_str(&format!("bx: {:#06x} ({})\n", self.bx.1, self.bx.1));
        s.push_str(&format!("cx: {:#06x} ({})\n", self.cx.1, self.cx.1));
        s.push_str(&format!("dx: {:#06x} ({})\n", self.dx.1, self.dx.1));
        s.push_str(&format!("sp: {:#06x} ({})\n", self.sp.1, self.sp.1));
        s.push_str(&format!("bp: {:#06x} ({})\n", self.bp.1, self.bp.1));
        s.push_str(&format!("si: {:#06x} ({})\n", self.si.1, self.si.1));
        s.push_str(&format!("di: {:#06x} ({})\n", self.di.1, self.di.1));
        write!(f, "{}", s)
    }
}
