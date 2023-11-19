use std::fs;
use rand::prelude::*;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

pub const FONT: [u8; 80] = [
    0xF0,
    0x90,
    0x90,
    0x90,
    0xF0,
    0x20,
    0x60,
    0x20,
    0x20,
    0x70,
    0xF0,
    0x10,
    0xF0,
    0x80,
    0xF0,
    0xF0,
    0x10,
    0xF0,
    0x10,
    0xF0,
    0x90,
    0x90,
    0xF0,
    0x10,
    0x10,
    0xF0,
    0x80,
    0xF0,
    0x10,
    0xF0,
    0xF0,
    0x80,
    0xF0,
    0x90,
    0xF0,
    0xF0,
    0x10,
    0x20,
    0x40,
    0x40,
    0xF0,
    0x90,
    0xF0,
    0x90,
    0xF0,
    0xF0,
    0x90,
    0xF0,
    0x10,
    0xF0,
    0xF0,
    0x90,
    0xF0,
    0x90,
    0x90,
    0xE0,
    0x90,
    0xE0,
    0x90,
    0xE0,
    0xF0,
    0x80,
    0x80,
    0x80,
    0xF0,
    0xE0,
    0x90,
    0x90,
    0x90,
    0xE0,
    0xF0,
    0x80,
    0xF0,
    0x80,
    0xF0,
    0xF0,
    0x80,
    0xF0,
    0x80,
    0x80,
];

pub struct Chip8 {
    memory: [u8; 4096],
    stack: [u16; 16],
    pub display: [[u8; WIDTH as usize]; HEIGHT as usize],
    v_registers: [u8; 16],
    i_register: u16,
    pc: u16,
    sp: u16,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut chip8 = Chip8 {
            memory: [0; 4096],
            stack: [0; 16],
            display: [[0; WIDTH as usize]; HEIGHT as usize],
            v_registers: [0; 16],
            i_register: 0,
            pc: 0x200,
            sp: 0
        };
        chip8.load_font();
        chip8
    }

    fn load_font(&mut self) {
        for i in 0..FONT.len() {
            self.memory[i] = FONT[i]; 
        }
    }

    fn fetch_instruction(&self) -> u16 {
        let mut instruction = (self.memory[self.pc as usize] as u16) << 8;
        instruction | self.memory[(self.pc + 1) as usize] as u16
    }

    fn execute_instruction(&mut self, instruction: u16) -> Result<(), String> {
        self.pc += 2;
        Ok(match instruction {
            0x00E0 => self.display = [[0; WIDTH as usize]; HEIGHT as usize],
            0xA000..=0xAFFF => self.i_register = instruction & 0x0FFF,
            0x6000..=0x6FFF => {
                let register_idx: u8 = ((instruction >> 8) as u8) & 0x0F;
                let register_value: u8 = (instruction & 0x00FF) as u8;
                self.v_registers[register_idx as usize] = register_value;
            },
            0xD000..=0xDFFF => {
                let x_coord = ((instruction >> 8) as u8) & 0x0F;
                let x_coord = self.v_registers[x_coord as usize];

                let y_coord = ((instruction >> 4) & 0x000F) as u8;
                let y_coord = self.v_registers[y_coord as usize];

                let height = (instruction & 0x000F) as u8;

                let mut any_set = false;

                for i in 0..height {
                    let row = self.memory[(self.i_register + (i as u16)) as usize];
                    for j in 0..8 {
                        if x_coord + j > 63 {
                            break
                        }

                        let bit_j = (row >> (7 - j)) & 1;

                        if bit_j != self.display[(y_coord + i) as usize][(x_coord + j) as usize] {
                            any_set = true;
                            self.display[(y_coord + i) as usize][(x_coord + j) as usize] = bit_j;
                        }
                    }
                }

                self.v_registers[0xF] = if any_set { 1 } else { 0 }; 
            },
            0x7000..=0x7FFF => {
                let register_idx = (instruction >> 8) & 0x000F;
                let value = instruction & 0x00FF;
                self.v_registers[register_idx as usize] = self.v_registers[register_idx as usize].wrapping_add(value as u8);
            },
            0x1000..=0x1FFF => self.pc = instruction & 0x0FFF,
            0x3000..=0x3FFF => {
                let register_idx = (instruction >> 8) & 0x000F;
                let value = (instruction & 0x00FF) as u8;
                if self.v_registers[register_idx as usize] == value {
                    self.pc += 2;
                }
            },
            0x4000..=0x4FFF => {
                let register_idx = (instruction >> 8) & 0x000F;
                let value = (instruction & 0x00FF) as u8;
                if self.v_registers[register_idx as usize] != value {
                    self.pc += 2;
                } 
            },
            0x5000..=0x5FFF => {
                match instruction & 0x000F {
                    0 => {
                        let x_coord = ((instruction >> 8) as u8) & 0x0F;
                        let x_coord = self.v_registers[x_coord as usize];
        
                        let y_coord = ((instruction >> 4) & 0x000F) as u8;
                        let y_coord = self.v_registers[y_coord as usize];            
        
                        if (x_coord == y_coord) {
                            self.pc += 2;
                        }
                    },
                    _ => return Err(format!("Opcode 0x{:04X} is not defined", instruction))
                }
            },
            0x9000..=0x9FF0 => {
                let x_coord = ((instruction >> 8) as u8) & 0x0F;
                let x_coord = self.v_registers[x_coord as usize];

                let y_coord = ((instruction >> 4) & 0x000F) as u8;
                let y_coord = self.v_registers[y_coord as usize];            

                if (x_coord != y_coord) {
                    self.pc += 2;
                }
            },
            0x2000..=0x2FFF => {
                let address = instruction & 0x0FFF;
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = address;
            },
            0x00EE => {
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
            },
            0x8000..=0x8FFF => {
                let x_coord = ((instruction >> 8) as u8) & 0x0F;

                let y_coord = ((instruction >> 4) & 0x000F) as u8;
                let y_coord = self.v_registers[y_coord as usize];           

                match instruction & 0x000F {
                    0 => self.v_registers[x_coord as usize] = y_coord,
                    1 => self.v_registers[x_coord as usize] |= y_coord,
                    2 => self.v_registers[x_coord as usize] &= y_coord,
                    3 => self.v_registers[x_coord as usize] ^= y_coord,
                    4 => {
                        let result = self.v_registers[x_coord as usize] as u16 + y_coord as u16;
                        self.v_registers[x_coord as usize] = result as u8;
                        self.v_registers[0xF] = if result > 0xFF { 1 } else { 0 };
                    }
                    5 => {
                        let borrow = if self.v_registers[x_coord as usize] < y_coord { 0 } else { 1 };
                        self.v_registers[x_coord as usize] = self.v_registers[x_coord as usize].wrapping_sub(y_coord);
                        self.v_registers[0xF] = borrow;
                    },
                    7 => {
                        let borrow = if self.v_registers[x_coord as usize] > y_coord { 0 } else { 1 };
                        self.v_registers[x_coord as usize] = y_coord.wrapping_sub(self.v_registers[x_coord as usize]);   
                        self.v_registers[0xF] = borrow;                    
                    },
                    6 => {
                        self.v_registers[0xF] = self.v_registers[x_coord as usize] & 1;
                        self.v_registers[x_coord as usize] >>= 1;
                    },
                    0xE => {
                        self.v_registers[0xF] = (self.v_registers[x_coord as usize] >> 7) & 1;
                        self.v_registers[x_coord as usize] <<= 1;
                    },
                    _ => return Err(format!("Opcode 0x{:04X} is not defined", instruction))
                };
            },
            0xF000..=0xFFFF => {
                match instruction & 0x00FF {
                    0x55 => {
                        let register_idx = (instruction >> 8) & 0x000F;
                        for idx in 0..=register_idx {
                            self.memory[(self.i_register + idx) as usize] = self.v_registers[register_idx as usize];
                        }
                    },
                    0x65 => {
                        let register_idx = (instruction >> 8) & 0x000F;
                        for idx in 0..=register_idx {
                            self.v_registers[register_idx as usize] = self.memory[(self.i_register + idx) as usize];
                        }
                    },
                    0x33 => {
                        let x = ((instruction >> 8) & 0x000F) as usize;
                        self.memory[self.i_register as usize] = self.v_registers[x] / 100;
                        self.memory[(self.i_register + 1) as usize] = (self.v_registers[x] / 10) % 10;
                        self.memory[(self.i_register + 2) as usize] = (self.v_registers[x] % 100) % 10;
                    },
                    0x29 => {
                        let x = ((instruction >> 8) & 0x000F) as usize;
                        self.i_register = (self.v_registers[x].wrapping_mul(5)) as u16;
                    },
                    0x1E => {
                         let x = ((instruction >> 8) & 0x000F) as usize;                       
                         self.i_register += (self.v_registers[x] as u16);
                    },
                    0x0A => {
                        self.pc -= 2
                    },
                    0x15 | 0x18 | 0x07 => {},
                    _ => return Err(format!("Opcode 0x{:04X} is not defined", instruction))
                }
            },
            0xE000..=0xEFFF => {
                match instruction & 0x00FF {
                    0x9E | 0xA1 => {},
                    _ => return Err(format!("Opcode 0x{:04X} is not defined", instruction))
                }
            },
            0xC000..=0xCFFF => {
                let register_idx = (instruction >> 8) & 0x000F;
                let value = ((instruction) & 0x00FF) as u8;
                self.v_registers[register_idx as usize] = rand::random::<u8>() & value;
            },
            0x0000 => { },
            _ => return Err(format!("Opcode 0x{:04X} is not defined", instruction))
        }) 
    }

    pub fn cycle(&mut self) {
        let instruction = self.fetch_instruction();
        self.execute_instruction(instruction).expect("opcode should be defined");
    }
    
    pub fn load_rom(&mut self, filename: &str) {
        if let Ok(contents) = fs::read(filename) {
            for (idx, byte) in contents.iter().enumerate() {
                self.memory[0x200 + idx] = *byte;
            } 
        }
    }
}