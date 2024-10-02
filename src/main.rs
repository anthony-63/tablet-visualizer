use std::io::Read;

use macroquad::prelude::*;
use miniquad::window::set_window_size;
use serde::Deserialize;
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetSystemMetrics, SYSTEM_METRICS_INDEX};
const SCALE: i32 = 2;
const WINDOW_WIDTH: i32 = 1920/SCALE;
const WINDOW_HEIGHT: i32 = 1920/SCALE;

fn window_conf() -> Conf {
    Conf {
        window_title: "Tablet Visualizer".to_owned(),
        fullscreen: false,
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        ..Default::default()
    }
}

struct Trail {
    points: Vec<(f32, Vec2)>,
    decay_rate: f32,
    color: Color,
    size: f32,
    is_image: bool,
    tex: Option<Texture2D>,
}

impl Trail {
    pub fn new(decay_rate: f32, color: Color, size: f32, is_image: bool, tex: Option<Texture2D>) -> Self {
        return Self {
            points: vec![],
            decay_rate,
            color,
            size,
            is_image,
            tex,
        }
    }

    pub fn update(&mut self) {
        for point in self.points.iter_mut() {
            point.0 -= self.decay_rate * get_frame_time();
        }
        self.points.retain(|&p| !(p.0 < 0.))
    }

    pub fn add_new(&mut self, x: f32, y: f32) {
        self.points.push((1.0, Vec2::new(x, y)));
    }

    pub fn draw(&self) {
        for point in self.points.iter() {
            if self.is_image && self.tex.is_some() {
                draw_texture_ex(&self.tex.as_ref().unwrap(), point.1.x - self.size / 2., point.1.y - self.size / 2., self.color, DrawTextureParams {
                    dest_size: Some(Vec2::new(self.size, self.size)),
                    ..Default::default()
                });
            } else {
                draw_circle(point.1.x, point.1.y, self.size, self.color);
            }

        }
    }
}

#[derive(Deserialize)]
struct Config {
    cursor: CursorConfig,
    background: BackgroundConfig,
    trail: TrailConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cursor: CursorConfig {
                color: [255, 0, 0, 255],
                image: false,
                image_path: String::new(),
                size: 4.,
            }, 
            background: BackgroundConfig {
                color: [0, 255, 0, 255],
                screen_size: [1920, 1080],
            },
            trail: TrailConfig {
                decay_rate: 1.,
                image: false,
                size: 2.,
                enabled: true,
                image_path: String::new(),
                color: [255, 0, 0, 255]
            }
        }
    }
}

#[derive(Deserialize)]
struct CursorConfig {
    image: bool,
    image_path: String,
    size: f32,
    color: [u8; 4]
}

#[derive(Deserialize)]
struct TrailConfig {
    decay_rate: f32,
    image: bool,
    color: [u8; 4],
    size: f32,
    image_path: String,
    enabled: bool,
}

#[derive(Deserialize)]
struct BackgroundConfig {
    color: [u8; 4],
    screen_size: [i32; 2]
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut error = None;
    let config = match std::fs::File::open("visualizer.toml") {
        Ok(mut f) => {
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();
            match toml::from_str(&s) {
                Ok(t) => t,
                Err(e) => {
                    error = Some(format!("Config Error, Using Default\nIssue: {:?}", e.message()));
                    Config::default()
                }, 
            }

        },
        Err(_) => {
            error = Some(String::from("File Error, Using Default\nIssue: Failed to find visualizer.toml"));
            Config::default()
        },
    };
    set_window_size((config.background.screen_size[0] / SCALE) as u32, (config.background.screen_size[1] / SCALE) as u32);
    
    let image = if config.trail.image {
        Some(Texture2D::from_image(&load_image(&config.trail.image_path).await.unwrap()))
    } else {
        None
    };

    let cursor_tex = if config.cursor.image {
        Some(Texture2D::from_image(&load_image(&config.cursor.image_path).await.unwrap()))
    } else {
        None
    };

    let mut trail = Trail::new(config.trail.decay_rate, Color::from_rgba(config.trail.color[0], config.trail.color[1], config.trail.color[2], config.trail.color[3]), config.trail.size, config.trail.image, image);
    loop {
        let mut cursor_pos: windows::Win32::Foundation::POINT = Default::default();
        unsafe{ GetCursorPos(&mut cursor_pos).unwrap() };

        clear_background(Color::from_rgba(config.background.color[0], config.background.color[1], config.background.color[2], config.background.color[3]));
        let screen_size = unsafe { (GetSystemMetrics(SYSTEM_METRICS_INDEX(16)), GetSystemMetrics(SYSTEM_METRICS_INDEX(17))) };
        let scale: (f32, f32) = (WINDOW_WIDTH as f32 / screen_size.0 as f32, WINDOW_HEIGHT as f32 / screen_size.1 as f32);
        let pos: (f32, f32) = (cursor_pos.x as f32 * scale.0, (cursor_pos.y as f32 * scale.0));
        if config.trail.enabled {
            trail.add_new(pos.0, pos.1);
            trail.update();
            trail.draw();
        }
        if error.is_some() {
            let msgs = error.as_ref().unwrap().split("\n").map(|e| e.to_string() ).collect::<Vec<String>>();
            let mut y = 30.;
            for msg in msgs {
                draw_text(msg.as_str(), 0., y, 30., RED);
                y += 30.;
            }
        }

        if config.cursor.image && cursor_tex.is_some() {
            draw_texture_ex(&cursor_tex.as_ref().unwrap(), pos.0 - config.cursor.size / 2., pos.1 - config.cursor.size / 2., Color::from_rgba(config.cursor.color[0], config.cursor.color[1], config.cursor.color[2], config.cursor.color[3]), DrawTextureParams {
                dest_size: Some(Vec2::new(config.cursor.size, config.cursor.size)),
                ..Default::default()
            });
        } else {
            draw_circle(pos.0, pos.1, config.cursor.size, Color::from_rgba(config.cursor.color[0], config.cursor.color[1], config.cursor.color[2], config.cursor.color[3]));
        }
        next_frame().await
    }
}