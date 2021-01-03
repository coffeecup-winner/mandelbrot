use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Point, render::Canvas, video::Window};
use std::time::Duration;

fn draw_mandelbrot(canvas: &mut Canvas<Window>) -> Result<(), String> {
    let (width, height) = canvas.output_size().unwrap();

    canvas.set_draw_color(Color::RGB(128, 128, 128));
    for x in 0..width {
        for y in 0..height {
            canvas.draw_point(Point::new(x as i32, y as i32))?;
        }
    }

    Ok(())
}

pub fn main() -> Result<(), String> {
    static WINDOW_NAME: &str = "Mandelbrot Explorer";
    const VIEWPORT_WIDTH: u32 = 800;
    const VIEWPORT_HEIGHT: u32 = 600;
    const FRAMES_PER_SECOND: u32 = 60;

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(WINDOW_NAME, VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;
    'main: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        draw_mandelbrot(&mut canvas)?;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                _ => {}
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FRAMES_PER_SECOND));
    }

    Ok(())
}
