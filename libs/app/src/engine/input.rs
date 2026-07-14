

use std::collections::HashSet;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::PhysicalKey;

pub struct InputState {
    pub keys_held:     HashSet<PhysicalKey>,
    pub keys_pressed:  HashSet<PhysicalKey>,  // only true for one frame
    pub keys_released: HashSet<PhysicalKey>,  // only true for one frame
    pub mouse_pos:     (f64, f64),
    pub mouse_delta:   (f64, f64),
    pub mouse_buttons: HashSet<MouseButton>,
    pub scroll_delta:  (f32, f32),
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys_held:     HashSet::new(),
            keys_pressed:  HashSet::new(),
            keys_released: HashSet::new(),
            mouse_pos:     (0.0, 0.0),
            mouse_delta:   (0.0, 0.0),
            mouse_buttons: HashSet::new(),
            scroll_delta:  (0.0, 0.0),
        }
    }

    /// Called at the start of each frame to clear per-frame state
    pub fn begin_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_delta  = (0.0, 0.0);
        self.scroll_delta = (0.0, 0.0);
    }

    /// Feed winit events into input state
    pub fn handle(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                tracing::trace!("Key {:?}: {:?}", event.physical_key, event.state);
                match event.state {
                    ElementState::Pressed => {
                        self.keys_pressed.insert(event.physical_key);
                        self.keys_held.insert(event.physical_key);
                    }
                    ElementState::Released => {
                        self.keys_released.insert(event.physical_key);
                        self.keys_held.remove(&event.physical_key);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = (position.x, position.y);
                self.mouse_delta = (
                    new_pos.0 - self.mouse_pos.0,
                    new_pos.1 - self.mouse_pos.1,
                );
                self.mouse_pos = new_pos;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed  => { self.mouse_buttons.insert(*button); }
                    ElementState::Released => { self.mouse_buttons.remove(button); }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                use winit::event::MouseScrollDelta;
                self.scroll_delta = match delta {
                    MouseScrollDelta::LineDelta(x, y)   => (*x, *y),
                    MouseScrollDelta::PixelDelta(pos)   => (pos.x as f32, pos.y as f32),
                };
            }
            _ => {}
        }
    }

    pub fn is_key_held(&self, key: PhysicalKey)     -> bool { self.keys_held.contains(&key) }
    pub fn is_key_pressed(&self, key: PhysicalKey)  -> bool { self.keys_pressed.contains(&key) }
    pub fn is_key_released(&self, key: PhysicalKey) -> bool { self.keys_released.contains(&key) }
    pub fn is_mouse_held(&self, btn: MouseButton)   -> bool { self.mouse_buttons.contains(&btn) }
}