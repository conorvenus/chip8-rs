use std::time::{Duration, Instant};

use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::event::Event;
use sdl2::render::Canvas;
use sdl2::video::Window;

mod chip8;
use chip8::Chip8;

const CLOCK_SPEED: f64 = 600.0;
const FPS: f64 = 60.0;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("CHIP8 EMULATOR", 640, 320)
        .fullscreen_desktop()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_logical_size(64, 32).unwrap();

    let mut emulator: Chip8 = Chip8::new();
    emulator.load_rom("roms/Pong (1 player).ch8")
        .expect("the file should exist");

    let mut last_render = Instant::now();

    'running: loop {
        let start_time = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode, .. } => emulator.on_key_down(keycode.unwrap()),
                Event::KeyUp { keycode, .. } => emulator.on_key_up(keycode.unwrap()),
                _ => ()
            }
        }

        emulator.cycle();

        if emulator.display_changed && (Instant::now() - last_render).as_secs_f64() >= 1.0 / FPS {
            render_display(&emulator, &mut canvas);
            last_render = Instant::now();
        }

        let time_elapsed = Instant::now() - start_time;
        let sleep_for = 1.0 / CLOCK_SPEED - time_elapsed.as_secs_f64();
        if sleep_for > 0.0 {
            spin_sleep::sleep(Duration::from_secs_f64(sleep_for));
        }
    }
}

fn clear_display(canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(255, 255, 255));
}

fn render_display(emulator: &Chip8, canvas: &mut Canvas<Window>) {
    clear_display(canvas);

    for (y, row) in emulator.display.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            if pixel == 1 {
                canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
            }
        }
    }

    canvas.present();
}