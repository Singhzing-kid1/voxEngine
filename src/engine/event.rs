use std::{
    time::{self, Duration},
};

use sdl3::{
    event::Event,
    keyboard::Keycode::{self},
};

use super::{Engine, RENDERMODE};

impl Engine {
    pub fn event_handling(&mut self) {
        self.x_offset = self.accum_x - self.last_x;
        self.y_offset = self.last_y - self.accum_y;

        self.last_x = self.accum_x;
        self.last_y = self.accum_y;

        self.collected_events = self.event.poll_iter().collect();

        for event in &self.collected_events {
            match event {
                Event::Quit { .. } => {
                    self.flags.set_quit_state(true);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::G),
                    ..
                } => {
                    self.flags.set_gravity_state(true);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::M),
                    ..
                } => {
                    self.flags
                        .set_capture_mouse_state(!self.flags.get_capture_mouse_state());
                    self.sdl_context.mouse().set_relative_mouse_mode(
                        &self.window,
                        self.flags.get_capture_mouse_state(),
                    );
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    let render_mode = self.current_render_mode as u32;
                    let new_render_mode = (render_mode + 1) % 6;
                    self.current_render_mode = match new_render_mode {
                        0 => RENDERMODE::DEFAULT,
                        1 => RENDERMODE::COORD,
                        2 => RENDERMODE::STEPS,
                        3 => RENDERMODE::NORMAL,
                        4 => RENDERMODE::UV,
                        5 => RENDERMODE::DEPTH,
                        _ => RENDERMODE::DEFAULT,
                    }
                }
                Event::MouseMotion {
                    xrel, yrel, x, y, ..
                } => {
                    if self.flags.get_capture_mouse_state() {
                        self.accum_x -= xrel;
                        self.accum_y += yrel;

                        self.mouse_x = *x;
                        self.mouse_y = *y;

                        let new_mouse_x = self.mouse_x + xrel;
                        let new_mouse_y = self.mouse_y + yrel;

                        if new_mouse_x < self.width as f32 / 2.0
                            || new_mouse_x > self.width as f32 / 2.0
                            || new_mouse_y < self.height as f32 / 2.0
                            || new_mouse_y > self.height as f32 / 2.0
                        {
                            self.sdl_context.mouse().warp_mouse_in_window(
                                &self.window,
                                self.width as f32 / 2.0,
                                self.height as f32 / 2.0,
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn frame_start(&mut self) {
        let current_frame = time::Instant::now();
        self.delta_time = (current_frame - self.last_frame).as_millis();
        self.last_frame = current_frame;
    }

    pub fn frame_end(&mut self, target_fps: i32) {
        if target_fps == -1 {
            return;
        }
        let target_dt = Duration::from_secs_f64(1.0 / target_fps as f64);
        let frame_duration = self.last_frame.elapsed();

        if frame_duration < target_dt {
            std::thread::sleep(target_dt - frame_duration);
        }
    }

    pub fn toggle_mouse(&self, toggle: bool) {
        self.sdl_context
            .mouse()
            .set_relative_mouse_mode(&self.window, toggle);
    }
}