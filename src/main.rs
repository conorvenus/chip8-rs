use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::event::Event;

mod chip8;
use chip8::Chip8;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut window = video_subsystem.window("CHIP8 EMULATOR", 640, 320)
        .position_centered()
        .fullscreen_desktop()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_logical_size(64, 32).unwrap();

    let mut emulator: Chip8 = Chip8::new();
    emulator.load_rom("roms/IBM Logo.ch8");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'running,
                _ => {}
            }
        }

        emulator.cycle();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (y, row) in emulator.display.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                if *pixel == 1 {
                    canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
                }
            }
        }

        canvas.present();
    }
}