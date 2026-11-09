use super::Engine;
use sdl3::{event::Event, keyboard::Keycode};
use std::time::{Duration, Instant};

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
                    self.flags.set_quit(true);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::G),
                    ..
                } => {
                    self.flags.set_gravity(true);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::M),
                    ..
                } => {
                    self.flags
                        .set_capture_mouse(!self.flags.get_capture_mouse());
                    self.context
                        .mouse()
                        .set_relative_mouse_mode(&self.window, self.flags.get_capture_mouse());
                }
                Event::MouseMotion {
                    xrel, yrel, x, y, ..
                } => {
                    if self.flags.get_capture_mouse() {
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
                            self.context.mouse().warp_mouse_in_window(
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
        let current_frame = Instant::now();
        self.delta_time = (current_frame - self.last_frame).as_secs_f64() * 1000f64;
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
        self.context
            .mouse()
            .set_relative_mouse_mode(&self.window, toggle);
    }

    pub fn start_text_input(&self) {
        self.context
            .video()
            .unwrap()
            .text_input()
            .start(&self.window);
    }

    pub fn stop_text_input(&self) {
        self.context
            .video()
            .unwrap()
            .text_input()
            .stop(&self.window);
    }
}
