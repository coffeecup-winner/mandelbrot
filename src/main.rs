use std::fs::File;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use palette::{Gradient, Hsv, LinSrgb};
use rayon::prelude::*;
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton, pixels::Color, rect::Point};

#[derive(Clone)]
struct Viewport {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    pixel_width: f64,
    pixel_height: f64,
}

impl Viewport {
    pub fn offset_x(&mut self, offset: f64) {
        self.x += self.width * offset;
    }

    pub fn offset_y(&mut self, offset: f64) {
        self.y += self.height * offset;
    }

    pub fn zoom_in(&mut self, factor: f64) {
        let offset_factor = factor / 2.0;
        self.x += self.width * offset_factor;
        self.y += self.height * offset_factor;
        self.height *= 1.0 - factor;
        self.width *= 1.0 - factor;
    }

    pub fn zoom_out(&mut self, factor: f64) {
        let offset_factor = factor / 2.0;
        self.height /= 1.0 - factor;
        self.width /= 1.0 - factor;
        self.x -= self.width * offset_factor;
        self.y -= self.height * offset_factor;
    }
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
    HslGradient,
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
        PaletteType::HslGradient => {
            let grad = Gradient::new(vec![
                Hsv::from(LinSrgb::new(0.1, 1.0, 1.0)),
                Hsv::from(LinSrgb::new(1.0, 0.1, 0.1)),
            ]);

            for color in grad.take(n as usize) {
                let c: LinSrgb = color.into();
                palette.push(Color::RGB(
                    (c.red * 255.0) as u8,
                    (c.green * 255.0) as u8,
                    (c.blue * 255.0) as u8,
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

    let palette = create_palette(1000, PaletteType::HslGradient);

    let mut pixel_buffer = vec![
        Pixel {
            x: 0,
            y: 0,
            color: Color::BLACK
        };
        (WINDOW_WIDTH * WINDOW_HEIGHT) as usize
    ];

    let mut i = 0;
    for y in 0..WINDOW_HEIGHT {
        for x in 0..WINDOW_WIDTH {
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
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;
    'main: loop {
        let mut needs_refresh = false;
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
                            viewport.offset_x(-0.1);
                        }
                        Keycode::D => {
                            viewport.offset_x(0.1);
                        }
                        Keycode::W => {
                            viewport.offset_y(-0.1);
                        }
                        Keycode::S => {
                            viewport.offset_y(0.1);
                        }
                        Keycode::E => {
                            viewport.zoom_in(0.2);
                        }
                        Keycode::Q => {
                            viewport.zoom_out(0.2);
                        }
                        Keycode::G => {
                            let filename = format!(
                                "mandelbrot_{}.gif",
                                SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .map_err(|e| e.to_string())?
                                    .as_secs()
                            );
                            println!("Writing {}... ", filename);
                            let mut image = File::create(&filename).map_err(|e| e.to_string())?;
                            let mut encoder = gif::Encoder::new(
                                &mut image,
                                WINDOW_WIDTH as u16,
                                WINDOW_HEIGHT as u16,
                                &[],
                            )
                            .map_err(|e| e.to_string())?;

                            let mut v = viewport.clone();
                            let mut buffer = pixel_buffer.clone();

                            while v.height < 4.0 {
                                draw_mandelbrot(&mut buffer, &v, &palette);
                                let mut pixels =
                                    Vec::with_capacity((3 * WINDOW_WIDTH * WINDOW_HEIGHT) as usize);
                                for p in &buffer {
                                    pixels.push(p.color.r);
                                    pixels.push(p.color.g);
                                    pixels.push(p.color.b);
                                }
                                let frame = gif::Frame::from_rgb_speed(
                                    WINDOW_WIDTH as u16,
                                    WINDOW_HEIGHT as u16,
                                    &pixels,
                                    10,
                                );
                                encoder.write_frame(&frame).map_err(|e| e.to_string())?;
                                v.zoom_out(0.1);
                            }
                            println!("Done!");
                        }
                        _ => {}
                    }
                    needs_refresh = true;
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    viewport.offset_x((x - WINDOW_WIDTH as i32 / 2) as f64 / WINDOW_WIDTH as f64);
                    viewport.offset_y((y - WINDOW_HEIGHT as i32 / 2) as f64 / WINDOW_HEIGHT as f64);
                    match mouse_btn {
                        MouseButton::Left => {
                            viewport.zoom_in(0.2);
                        }
                        MouseButton::Right => {
                            viewport.zoom_out(0.2);
                        }
                        _ => {}
                    }
                    needs_refresh = true;
                }
                _ => {}
            }
        }

        if needs_refresh {
            draw_mandelbrot(&mut pixel_buffer, &viewport, &palette);
            for pixel in &pixel_buffer {
                canvas.set_draw_color(pixel.color);
                canvas.draw_point(Point::new(pixel.x, pixel.y))?;
            }
            canvas.present();
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FRAMES_PER_SECOND));
    }

    Ok(())
}
