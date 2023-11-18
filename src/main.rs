mod chip8;
use chip8::Chip8;

pub fn main() {
    let mut emulator: Chip8 = Chip8::new();
    loop {
        emulator.cycle();
    }
}