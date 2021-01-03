use std::time::Duration;

use rayon::prelude::*;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Point};

struct Viewport {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    pixel_width: f64,
    pixel_height: f64,
}

#[derive(Clone, Copy)]
struct Pixel {
    x: i32,
    y: i32,
    color: Color,
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

fn draw_mandelbrot(pixel_buffer: &mut [Pixel], viewport: &Viewport, palette: &[Color]) {
    pixel_buffer.par_iter_mut().for_each(|pixel| {
        let r =
            viewport.x + ((pixel.x as f64 + 0.5) / viewport.pixel_width as f64) * viewport.width;
        let i =
            viewport.y + ((pixel.y as f64 + 0.5) / viewport.pixel_height as f64) * viewport.height;
        if !is_in_cardioid(r, i) && !is_in_period2_bulb(r, i) {
            let iter = iterate_point(r, i, 1000);
            if iter != 1000 {
                pixel.color = palette[iter as usize];
            } else {
                pixel.color = Color::BLACK;
            }
        } else {
            pixel.color = Color::BLACK;
        }
    });
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
    const WINDOW_WIDTH: u32 = 1050;
    const WINDOW_HEIGHT: u32 = 600;
    const FRAMES_PER_SECOND: u32 = 60;

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(WINDOW_NAME, WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut viewport = Viewport {
        x: -2.5,
        y: -1.0,
        width: 3.5,
        height: 2.0,
        pixel_width: WINDOW_WIDTH as f64,
        pixel_height: WINDOW_HEIGHT as f64,
    };

    let palette = create_palette(1000, PaletteType::PseudoRandom);

    let mut pixel_buffer = vec![
        Pixel {
            x: 0,
            y: 0,
            color: Color::BLACK
        };
        (WINDOW_WIDTH * WINDOW_HEIGHT) as usize
    ];

    let mut i = 0;
    for x in 0..WINDOW_WIDTH {
        for y in 0..WINDOW_HEIGHT {
            pixel_buffer[i].x = x as i32;
            pixel_buffer[i].y = y as i32;
            i += 1;
        }
    }

    draw_mandelbrot(&mut pixel_buffer, &viewport, &palette);
    for pixel in &pixel_buffer {
        canvas.set_draw_color(pixel.color);
        canvas.draw_point(Point::new(pixel.x, pixel.y))?;
    }

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
                    draw_mandelbrot(&mut pixel_buffer, &viewport, &palette);
                    for pixel in &pixel_buffer {
                        canvas.set_draw_color(pixel.color);
                        canvas.draw_point(Point::new(pixel.x, pixel.y))?;
                    }
                }
                _ => {}
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FRAMES_PER_SECOND));
    }

    Ok(())
}
