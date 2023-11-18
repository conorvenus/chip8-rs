use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

struct Chip8 {
    memory: [u8; 4096],
    stack: [u16; 16],
    display: [[u8; WIDTH as usize]; HEIGHT as usize],
    v_registers: [u8; 16],
    i_register: u16,
    pc: u16,
    sp: u16,
}

impl Chip8 {
    fn fetch_instruction(&self) -> u16 {
        let mut instruction = (self.memory[self.pc as usize] as u16) << 8;
        instruction | self.memory[(self.pc + 1) as usize] as u16
    }

    fn cycle(&mut self) {
        let instruction = self.fetch_instruction();
        self.pc += 2;
    }
}

pub fn main() {

}