use crate::decode::{self, Instruction, RegisterMemory, RegisterName};

pub struct Register(RegisterName, u16);

#[derive(Clone, Debug, PartialEq)]
pub struct Flags {
    sf: bool,
    zf: bool,
}

impl std::fmt::Display for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if self.sf {
            s.push('S');
        }
        if self.zf {
            s.push('Z');
        }
        write!(f, "{}", s)
    }
}

pub struct Hack86 {
    cpu: CPU,
    instructions: Vec<u8>,
    memory: [u8; 65536],
}

pub struct CPU {
    ax: Register,
    cx: Register,
    dx: Register,
    bx: Register,
    sp: Register,
    bp: Register,
    si: Register,
    di: Register,

    ip: u16,

    flags: Flags,

    cycle_count: u32,
}

impl Hack86 {
    pub fn new(instructions: Vec<u8>) -> Hack86 {
        Hack86 {
            cpu: CPU::new(),
            instructions,
            memory: [0; 65536],
        }
    }

    pub fn simulate(&mut self) {
        while usize::from(self.cpu.ip) < self.instructions.len() {
            if let Ok(instruction) =
                decode::decode_instruction(&self.instructions[usize::from(self.cpu.ip)..])
            {
                self.cpu.ip += u16::from(instruction.length);
                self.cpu.execute(&instruction, &mut self.memory);
            } else {
                panic!(
                    "unsupported instruction {:#010b} at offset {}",
                    self.instructions[usize::from(self.cpu.ip)],
                    self.cpu.ip
                );
            }
        }

        println!();
        println!("Final registers:");
        println!("{}", self.cpu);
    }

    pub fn memory(&self) -> &[u8] {
        &self.memory
    }
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            ax: Register(RegisterName::AX, 0),
            bx: Register(RegisterName::BX, 0),
            cx: Register(RegisterName::CX, 0),
            dx: Register(RegisterName::DX, 0),
            bp: Register(RegisterName::BP, 0),
            sp: Register(RegisterName::SP, 0),
            di: Register(RegisterName::DI, 0),
            si: Register(RegisterName::SI, 0),

            ip: 0,

            flags: Flags {
                sf: false,
                zf: false,
            },

            cycle_count: 0,
        }
    }

    pub fn execute(&mut self, instruction: &Instruction, memory: &mut [u8; 65536]) {
        print!("{} ;", instruction.instruction_category);

        let clocks = instruction.clocks();
        self.cycle_count += u32::from(clocks.0 + clocks.1);
        if clocks.1 > 0 {
            print!(
                " Clocks: +{} = {} ({} + {}ea) |",
                (clocks.0 + clocks.1),
                self.cycle_count,
                clocks.0,
                clocks.1
            );
        } else {
            print!(
                " Clocks: +{} = {} |",
                (clocks.0 + clocks.1),
                self.cycle_count
            );
        }

        let original_flags = self.flags.clone();

        match &instruction.instruction_category {
            decode::InstructionCategory::RegisterMemoryAndRegister(mnemonic, src, dest) => {
                match mnemonic {
                    decode::Mnemonic::MOV => match (src, dest) {
                        (
                            RegisterMemory::Register(src_name),
                            RegisterMemory::Register(dest_name),
                        ) => {
                            let new_value = self.get_register(src_name).1;
                            self.set_register(dest_name, new_value, false);
                        }
                        (
                            RegisterMemory::DirectAddress(address),
                            RegisterMemory::Register(dest_name),
                        ) => {
                            let value: u16 = u16::from_le_bytes([
                                memory[*address as usize],
                                memory[(*address + 1) as usize],
                            ]);
                            self.set_register(dest_name, value, true);
                        }
                        (
                            RegisterMemory::Register(src_name),
                            RegisterMemory::RegisterAddressOffset(dest_name, offset_name),
                        ) => {
                            let src_value = self.get_register(src_name).1;
                            let mut address = self.get_register(dest_name).1;
                            let offset = self.get_register(offset_name).1;
                            address += offset;
                            let bytes = src_value.to_le_bytes();
                            memory[address as usize] = bytes[0];
                            memory[(address + 1) as usize] = bytes[1];
                        }
                        (
                            RegisterMemory::RegisterAddressOffset(src_name, offset_name),
                            RegisterMemory::Register(dest_name),
                        ) => {
                            let mut address = self.get_register(src_name).1;
                            let offset = self.get_register(offset_name).1;
                            address += offset;
                            let value = u16::from_le_bytes([
                                memory[address as usize],
                                memory[(address + 1) as usize],
                            ]);
                            self.set_register(dest_name, value, true);
                        }
                        (
                            RegisterMemory::RegisterAddressDisplacement(src_name, displacement),
                            RegisterMemory::Register(dest_name),
                        ) => {
                            let address = self.get_register(src_name).1 + displacement;
                            let value = u16::from_le_bytes([
                                memory[address as usize],
                                memory[(address + 1) as usize],
                            ]);
                            self.set_register(dest_name, value, true);
                        }
                        (
                            RegisterMemory::Register(src_name),
                            RegisterMemory::RegisterAddressDisplacement(dest_name, displacement),
                        ) => {
                            let value = self.get_register(src_name).1;
                            let address = self.get_register(dest_name).1 + displacement;
                            let bytes = value.to_le_bytes();
                            memory[address as usize] = bytes[0];
                            memory[(address + 1) as usize] = bytes[1];
                        }
                        (
                            RegisterMemory::RegisterAddress(src_name),
                            RegisterMemory::Register(dest_name),
                        ) => {
                            let address = self.get_register(src_name).1;
                            let value = u16::from_le_bytes([
                                memory[address as usize],
                                memory[(address + 1) as usize],
                            ]);
                            self.set_register(dest_name, value, true);
                        }
                        (
                            RegisterMemory::Register(src_name),
                            RegisterMemory::RegisterAddress(dest_name),
                        ) => {
                            let value = self.get_register(src_name).1;
                            let address = self.get_register(dest_name).1;
                            let bytes = value.to_le_bytes();
                            memory[address as usize] = bytes[0];
                            memory[(address + 1) as usize] = bytes[1];
                        }
                        _ => todo!(),
                    },
                    decode::Mnemonic::ADD => match (src, dest) {
                        (
                            RegisterMemory::Register(src_name),
                            RegisterMemory::Register(dest_name),
                        ) => {
                            let new_value = self
                                .get_register(dest_name)
                                .1
                                .overflowing_add(self.get_register(src_name).1)
                                .0;
                            self.set_register(dest_name, new_value, true);
                        }
                        (
                            RegisterMemory::Register(src_name),
                            RegisterMemory::RegisterAddressDisplacement(dest_name, displacement),
                        ) => {
                            let a = self.get_register(src_name).1;
                            let address = self.get_register(dest_name).1 + displacement;
                            let b = u16::from_le_bytes([
                                memory[address as usize],
                                memory[(address + 1) as usize],
                            ]);

                            let value = a.overflowing_add(b).0;

                            let bytes = value.to_le_bytes();
                            memory[address as usize] = bytes[0];
                            memory[(address + 1) as usize] = bytes[1];
                        }
                        _ => todo!(),
                    },
                    decode::Mnemonic::SUB => match (src, dest) {
                        (
                            RegisterMemory::Register(src_name),
                            RegisterMemory::Register(dest_name),
                        ) => {
                            let new_value = self
                                .get_register(dest_name)
                                .1
                                .overflowing_sub(self.get_register(src_name).1)
                                .0;
                            self.set_register(dest_name, new_value, true);
                        }
                        _ => todo!(),
                    },
                    decode::Mnemonic::CMP => match (src, dest) {
                        (
                            RegisterMemory::Register(src_name),
                            RegisterMemory::Register(dest_name),
                        ) => {
                            let a = self.get_register(src_name).1;
                            let b = self.get_register(dest_name).1;
                            self.update_flags(a, b);
                        }
                        _ => todo!(),
                    },
                    _ => todo!(),
                };
            }
            decode::InstructionCategory::ImmediateToRegister(mnemonic, immediate, dest) => {
                match mnemonic {
                    decode::Mnemonic::MOV => self.set_register(dest, *immediate, false),
                    _ => todo!(),
                };
            }
            decode::InstructionCategory::ImmediateToRegisterMemory(
                mnemonic,
                immediate,
                dest,
                _word_operation,
            ) => match mnemonic {
                decode::Mnemonic::ADD => match dest {
                    RegisterMemory::Register(dest_name) => {
                        let dest_value = self.get_register(dest_name).1;
                        self.set_register(dest_name, dest_value + *immediate, true);
                    }
                    _ => todo!(),
                },
                decode::Mnemonic::SUB => match dest {
                    RegisterMemory::Register(dest_name) => {
                        let dest_value = self.get_register(dest_name).1;
                        self.set_register(dest_name, dest_value - *immediate, true);
                    }
                    _ => todo!(),
                },
                decode::Mnemonic::MOV => match dest {
                    RegisterMemory::DirectAddress(address) => {
                        let bytes = immediate.to_le_bytes();
                        memory[*address as usize] = bytes[0];
                        memory[(*address + 1) as usize] = bytes[1];
                    }
                    RegisterMemory::RegisterAddressDisplacement(dest_name, displacement) => {
                        let mut address = self.get_register(dest_name).1;
                        address += displacement;
                        let bytes = immediate.to_le_bytes();
                        memory[address as usize] = bytes[0];
                        memory[(address + 1) as usize] = bytes[1];
                    }
                    _ => todo!(),
                },
                decode::Mnemonic::CMP => match dest {
                    RegisterMemory::Register(dest_name) => {
                        let dest_value = self.get_register(dest_name).1;
                        self.update_flags(*immediate, dest_value);
                    }
                    _ => todo!(),
                },
                _ => todo!(),
            },
            decode::InstructionCategory::ImmediateToAccumulator(_mnemonic, _immediate, _dest) => {
                todo!()
            }
            decode::InstructionCategory::Jump(mnemonic, increment) => match mnemonic {
                decode::Mnemonic::JNZ => {
                    if !self.flags.zf {
                        let increment = i16::from(*increment);
                        let new_ip = self
                            .ip
                            .checked_add_signed(increment.into())
                            .expect("jnz increment should not overflow instruction pointer");
                        self.ip = new_ip;
                    }
                }
                _ => todo!(),
            },
        };

        let original_ip = self.ip - u16::from(instruction.length);
        print!(" ip:{:#x}->{:#x}", original_ip, self.ip);
        if original_flags != self.flags {
            println!(" flags:{}->{}", original_flags, self.flags)
        } else {
            println!();
        }
    }

    fn get_register(&mut self, name: &RegisterName) -> &mut Register {
        match name {
            RegisterName::AX => &mut self.ax,
            RegisterName::BX => &mut self.bx,
            RegisterName::CX => &mut self.cx,
            RegisterName::DX => &mut self.dx,
            RegisterName::BP => &mut self.bp,
            RegisterName::SP => &mut self.sp,
            RegisterName::DI => &mut self.di,
            RegisterName::SI => &mut self.si,
            _ => todo!(),
        }
    }

    fn set_register(&mut self, dest: &RegisterName, value: u16, set_flags: bool) {
        let register = self.get_register(dest);
        let prev = register.1;
        register.1 = value;

        let new_flags = Flags {
            sf: (value & 0x8000) != 0,
            zf: value == 0,
        };

        if set_flags && self.flags != new_flags {
            self.flags = new_flags;
        }

        if prev != value {
            print!(" {}:{:#x}->{:#x}", dest, prev, value);
        }
    }

    fn update_flags(&mut self, a: u16, b: u16) {
        let result = b.overflowing_sub(a).0;

        let new_flags = Flags {
            sf: (result & 0x8000) != 0,
            zf: result == 0,
        };

        if self.flags != new_flags {
            self.flags = new_flags;
        }
    }
}

impl std::fmt::Display for CPU {
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
        s.push_str(&format!("ip: {:#06x} ({})\n", self.ip, self.ip));
        s.push_str(&format!("flags: {}", self.flags));
        write!(f, "{}", s)
    }
}
