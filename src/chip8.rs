use std::{fs, any};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

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
        Chip8 {
            memory: [0; 4096],
            stack: [0; 16],
            display: [[0; WIDTH as usize]; HEIGHT as usize],
            v_registers: [0; 16],
            i_register: 0,
            pc: 0x200,
            sp: 0
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
                self.v_registers[register_idx as usize] += value as u8;
            },
            0x1000..=0x1FFF => self.pc = instruction & 0x0FFF,
            other => return Err(format!("Opcode 0x{:04X} is not defined", other))
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