use macroquad::{prelude::*, window};
use miniquad::window::screen_size;
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetSystemMetrics, SYSTEM_METRICS_INDEX};

const WINDOW_WIDTH: i32 = 1920/4;
const WINDOW_HEIGHT: i32 = 1920/4;

fn window_conf() -> Conf {
    Conf {
        window_title: "Tablet Visualizer".to_owned(),
        fullscreen: false,
        window_width: WINDOW_WIDTH,
        window_height: 1080/4,
        ..Default::default()
    }
}

struct Trail {
    points: Vec<(f32, Vec2)>,
    decay_rate: f32,
}

impl Trail {
    pub fn new(decay_rate: f32) -> Self {
        return Self {
            points: vec![],
            decay_rate
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
            draw_circle(point.1.x, point.1.y, 2., Color::from_rgba(255, 0, 0, (point.0 * 255.) as u8));
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut trail = Trail::new(1.);
    loop {
        let mut cursor_pos: windows::Win32::Foundation::POINT = Default::default();
        unsafe{ GetCursorPos(&mut cursor_pos).unwrap() };

        clear_background(WHITE);
        let screen_size = unsafe { (GetSystemMetrics(SYSTEM_METRICS_INDEX(16)), GetSystemMetrics(SYSTEM_METRICS_INDEX(17))) };
        let scale: (f32, f32) = (WINDOW_WIDTH as f32 / screen_size.0 as f32, WINDOW_HEIGHT as f32 / screen_size.1 as f32);
        let pos: (f32, f32) = (cursor_pos.x as f32 * scale.0, (cursor_pos.y as f32 * scale.0));
        draw_circle(pos.0, pos.1, 2., RED);
        trail.add_new(pos.0, pos.1);
        trail.update();
        trail.draw();
        next_frame().await
    }
}