use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Point, render::Canvas, video::Window,
};
use std::time::Duration;

struct Viewport {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn iterate_point(cr: f64, ci: f64, max_iter: u32) -> u32 {
    let mut iter = 0;
    let mut r = 0.0;
    let mut i = 0.0;
    let mut r2 = 0.0;
    let mut i2 = 0.0;
    while r2 + i2 < 4.0 && iter < max_iter {
        i = (r + r) * i + ci;
        r = r2 - i2 + cr;
        r2 = r * r;
        i2 = i * i;
        iter += 1;
    }
    iter
}

fn is_in_cardioid(r: f64, i: f64) -> bool {
    let p = ((r - 1.0 / 4.0).powi(2) + i.powi(2)).sqrt();
    r <= (p - 2.0 * p.powi(2) + 1.0 / 4.0)
}

fn is_in_period2_bulb(r: f64, i: f64) -> bool {
    ((r + 1.0).powi(2) + i.powi(2)) <= (1.0 / 16.0)
}

fn draw_mandelbrot(
    canvas: &mut Canvas<Window>,
    viewport: &Viewport,
    palette: &[Color],
) -> Result<(), String> {
    let (width, height) = canvas.output_size().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    for x in 0..width {
        for y in 0..height {
            let r = viewport.x + ((x as f64 + 0.5) / width as f64) * viewport.width;
            let i = viewport.y + ((y as f64 + 0.5) / height as f64) * viewport.height;
            if !is_in_cardioid(r, i) && !is_in_period2_bulb(r, i) {
                match iterate_point(r, i, 1000) {
                    1000 => {}
                    iter => {
                        canvas.set_draw_color(palette[iter as usize]);
                        canvas.draw_point(Point::new(x as i32, y as i32))?;
                    }
                };
            }
        }
    }

    Ok(())
}

enum PaletteType {
    PseudoRandom,
}

fn create_palette(n: u32, type_: PaletteType) -> Vec<Color> {
    let mut palette = vec![];
    match type_ {
        PaletteType::PseudoRandom => {
            for i in 0..n {
                palette.push(Color::RGB(
                    (i * 1337) as u8,
                    (i * 173) as u8,
                    (i * 6101) as u8,
                ));
            }
        }
    }
    palette
}

pub fn main() -> Result<(), String> {
    static WINDOW_NAME: &str = "Mandelbrot Explorer";
    const VIEWPORT_WIDTH: u32 = 1050;
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
    let mut viewport = Viewport {
        x: -2.5,
        y: -1.0,
        width: 3.5,
        height: 2.0,
    };

    let palette = create_palette(1000, PaletteType::PseudoRandom);
    draw_mandelbrot(&mut canvas, &viewport, &palette)?;

    let mut event_pump = sdl_context.event_pump()?;
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    match code {
                        Keycode::A => {
                            viewport.x -= viewport.width / 100.0;
                        }
                        Keycode::D => {
                            viewport.x += viewport.width / 100.0;
                        }
                        Keycode::W => {
                            viewport.y -= viewport.height / 100.0;
                        }
                        Keycode::S => {
                            viewport.y += viewport.height / 100.0;
                        }
                        Keycode::E => {
                            viewport.x += viewport.width / 10.0;
                            viewport.y += viewport.height / 10.0;
                            viewport.height *= 0.8;
                            viewport.width *= 0.8;
                        }
                        Keycode::Q => {
                            viewport.height /= 0.8;
                            viewport.width /= 0.8;
                            viewport.x -= viewport.width / 10.0;
                            viewport.y -= viewport.height / 10.0;
                        }
                        _ => {}
                    }
                    draw_mandelbrot(&mut canvas, &viewport, &palette)?;
                }
                _ => {}
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FRAMES_PER_SECOND));
    }

    Ok(())
}
