use std::fs;
use rand::Rng;

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

struct Instruction {
    instruction: u16,
    x: usize,
    y: usize,
    nnn: u16,
    nn: u8,
    n: u8
}

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
        let instruction = (self.memory[self.pc as usize] as u16) << 8;
        instruction | self.memory[(self.pc + 1) as usize] as u16
    }

    fn decode_instruction(&self, instruction: u16) -> Instruction {
        Instruction {
            instruction,
            x: ((instruction >> 8) & 0x000F) as usize,
            y: ((instruction >> 4) & 0x000F) as usize,
            nnn: instruction & 0x0FFF,
            nn: (instruction & 0x00FF) as u8,
            n: (instruction & 0x000F) as u8
        }
    }

    fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), String> {
        self.pc += 2;
        Ok(match instruction.instruction {
            0x00E0 => self.op_0x00e0(), 
            0x00EE => self.op_0x000e(),
            0x6000..=0x6FFF => self.op_0x6xnn(instruction),
            0xA000..=0xAFFF => self.op_0xannn(instruction),
            0xD000..=0xDFFF => self.op_0xdxyn(instruction),
            0x2000..=0x2FFF => self.op_0x2nnn(instruction),
            0x7000..=0x7FFF => self.op_0x7xnn(instruction),
            0x1000..=0x1FFF => self.op_0x1nnn(instruction),
            0x3000..=0x3FFF => self.op_0x3xnn(instruction),
            0xC000..=0xCFFF => self.op_0xcxnn(instruction),
            0x4000..=0x4FFF => self.op_0x4xnn(instruction),
            0x5000..=0x5FFF => self.op_0x5xy0(instruction),
            0x9000..=0x9FFF => self.op_0x9xy0(instruction),
            0xF000..=0xFFFF => {
                match instruction.nn {
                    0x1E => self.op_0xfx1e(instruction),
                    0x15 => self.op_0xfx15(instruction),
                    0x33 => self.op_0xfx33(instruction),
                    0x65 => self.op_0xfx65(instruction),
                    0x55 => self.op_0xfx55(instruction),
                    0x0A => self.op_0xfx0a(instruction),
                    0x07 => self.op_0xfx07(instruction),
                    0x29 => self.op_0xfx29(instruction),
                    0x18 => self.op_0xfx18(instruction),
                    _ => return Err(format!("Opcode 0x{:04X} is not defined", instruction.instruction))
                }
            },
            0x8000..=0x8FFF => {
                match instruction.n {
                    0x0 => self.op_0x8xy0(instruction),
                    0x1 => self.op_0x8xy1(instruction),
                    0x2 => self.op_0x8xy2(instruction),
                    0x3 => self.op_0x8xy3(instruction),
                    0x4 => self.op_0x8xy4(instruction),
                    0x5 => self.op_0x8xy5(instruction),
                    0x6 => self.op_0x8xy6(instruction),
                    0xE => self.op_0x8xye(instruction),
                    _ => return Err(format!("Opcode 0x{:04X} is not defined", instruction.instruction))
                }
            },
            0xE000..=0xEFFF => {
                match instruction.nn {
                    0xA1 => self.op_0xexa1(instruction),
                    0x9E => self.op_0xex9e(instruction),
                    _ => return Err(format!("Opcode 0x{:04X} is not defined", instruction.instruction))
                }
            },
            _ => return Err(format!("Opcode 0x{:04X} is not defined", instruction.instruction))
        })
    }

    pub fn cycle(&mut self) {
        let instruction = self.fetch_instruction();
        let instruction = self.decode_instruction(instruction);
        self.execute_instruction(instruction).expect("opcode should be defined");
    }
    
    pub fn load_rom(&mut self, filename: &str) {
        if let Ok(contents) = fs::read(filename) {
            for (idx, &byte) in contents.iter().enumerate() {
                self.memory[0x200 + idx] = byte;
            } 
        }
    }

    fn op_0x00e0(&mut self) {
        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                self.display[i as usize][j as usize] = 0;
            }
        }
    }

    fn op_0x6xnn(&mut self, instruction: Instruction) {
        self.v_registers[instruction.x] = instruction.nn;
    }

    fn op_0xannn(&mut self, instruction: Instruction) {
        self.i_register = instruction.nnn;
    }

    fn op_0xdxyn(&mut self, instruction: Instruction) {
        let (x, y) = (self.v_registers[instruction.x], self.v_registers[instruction.y]);
        for i in 0..instruction.n {
            let bits = self.memory[(self.i_register + i as u16) as usize];
            for j in 0..8 {
                self.display[((y + i) as usize) % HEIGHT as usize][((x + j) as usize) % WIDTH as usize] = (bits >> (7-j)) & 1;
            }
        }
    }

    fn op_0x2nnn(&mut self, instruction: Instruction) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = instruction.nnn;
    }

    fn op_0x7xnn(&mut self, instruction: Instruction) {
        self.v_registers[instruction.x] = self.v_registers[instruction.x].wrapping_add(instruction.nn);
    }

    fn op_0x1nnn(&mut self, instruction: Instruction) {
        self.pc = instruction.nnn;
    }

    fn op_0x000e(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    fn op_0x3xnn(&mut self, instruction: Instruction) {
        if self.v_registers[instruction.x] == instruction.nn {
            self.pc += 2;
        }
    }

    fn op_0xcxnn(&mut self, instruction: Instruction) {
        let rand_value: u8 = rand::thread_rng().gen::<u8>();
        self.v_registers[instruction.x] = rand_value & instruction.nn;
    }

    fn op_0x4xnn(&mut self, instruction: Instruction) {
        if self.v_registers[instruction.x] != instruction.nn {
            self.pc += 2;
        }
    }

    fn op_0xfx1e(&mut self, instruction: Instruction) {
        self.i_register = self.i_register.wrapping_add(self.v_registers[instruction.x] as u16);
    }

    fn op_0xfx15(&mut self, instruction: Instruction) {
        // TODO
    }

    fn op_0xfx18(&mut self, instruction: Instruction) {
        // TODO
    }

    fn op_0xfx0a(&mut self, instruction: Instruction) {
        // TODO
        self.pc -= 2;
    }

    fn op_0xfx07(&mut self, instruction: Instruction) {
        // TODO
        self.v_registers[instruction.x] = 0;
    }

    fn op_0x8xy0(&mut self, instruction: Instruction) {
        self.v_registers[instruction.x] = self.v_registers[instruction.y];
    }

    fn op_0x8xy1(&mut self, instruction: Instruction) {
        self.v_registers[instruction.x] |= self.v_registers[instruction.y];
    }

    fn op_0x8xy2(&mut self, instruction: Instruction) {
        self.v_registers[instruction.x] &= self.v_registers[instruction.y];
    }

    fn op_0x8xy3(&mut self, instruction: Instruction) {
        self.v_registers[instruction.x] ^= self.v_registers[instruction.y];
    }

    fn op_0x8xy4(&mut self, instruction: Instruction) {
        let (x, y) = (self.v_registers[instruction.x] as u16, self.v_registers[instruction.y] as u16);
        self.v_registers[0xF] = if x + y > 0xFF { 1 } else { 0 };
        self.v_registers[instruction.x] = self.v_registers[instruction.x].wrapping_add(y as u8);
    }

    fn op_0x8xy5(&mut self, instruction: Instruction) {
        let (x, y) = (self.v_registers[instruction.x], self.v_registers[instruction.y]);
        self.v_registers[0xF] = if y > x { 0 } else { 1 };
        self.v_registers[instruction.x] = self.v_registers[instruction.x].wrapping_sub(y);
    }

    fn op_0x8xy6(&mut self, instruction: Instruction) {
        self.v_registers[0xF] = self.v_registers[instruction.x] & 1;
        self.v_registers[instruction.x] >>= 1;
    }
    
    fn op_0x8xye(&mut self, instruction: Instruction) {
        self.v_registers[0xF] = (self.v_registers[instruction.x] >> 7) & 1;
        self.v_registers[instruction.x] <<= 1;
    }

    fn op_0xfx33(&mut self, instruction: Instruction) {
        let vx = self.v_registers[instruction.x];
        self.memory[self.i_register as usize] = vx / 100;
        self.memory[self.i_register as usize + 1] = (vx / 10) % 10;  
        self.memory[self.i_register as usize + 2] = vx % 10;
    }

    fn op_0xfx65(&mut self, instruction: Instruction) {
        for i in 0..=instruction.x {
            self.v_registers[i] = self.memory[self.i_register as usize + i];
        }
    }

    fn op_0xfx55(&mut self, instruction: Instruction) {
        for i in 0..=instruction.x {
            self.memory[self.i_register as usize + i] = self.v_registers[i];
        }
    }

    fn op_0xfx29(&mut self, instruction: Instruction) {
        self.i_register = self.v_registers[instruction.x] as u16 * 5; 
    }

    fn op_0xex9e(&mut self, instruction: Instruction) {

    }

    fn op_0xexa1(&mut self, instruction: Instruction) {
        self.pc += 2;
    }

    fn op_0x5xy0(&mut self, instruction: Instruction) {
        if self.v_registers[instruction.x] == self.v_registers[instruction.y] {
            self.pc += 2;
        }
    }

    fn op_0x9xy0(&mut self, instruction: Instruction) {
        if self.v_registers[instruction.x] != self.v_registers[instruction.y] {
            self.pc += 2;
        }
    }
}