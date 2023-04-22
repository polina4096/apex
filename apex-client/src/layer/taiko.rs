use std::time::Duration;

use cgmath::{vec3, vec2, Vector2};
use wcore::{audio::Audio, clock::{SyncClock, Clock}, time::Time, graphics::{context::Graphics, camera::{Projection, Camera}, layer::Layer}, color::Color};
use winit::dpi::PhysicalSize;

use crate::{taiko::{parser::Beatmap}, graphics::taiko::{conveyor::Conveyor}};


pub struct TaikoState {
    // Settings
    pub scale        : f32,
    pub audio_offset : i64, // ms
    pub hit_position : Vector2<f32>,
    pub zoom         : f32,
    pub don_color    : Color,
    pub kat_color    : Color,
    
    // Debug
    pub force_rebuild : bool,

    // Temporary
    pub hit_circles : bool,

    // Internal
    pub rebuild_pending : bool,
}

impl TaikoState {
    pub fn new() -> Self {
        return Self {
            scale        : 0.85,
            audio_offset : 65, // osu audio engine (bass) is oof...
            hit_position : vec2(300.0, 300.0),
            zoom         : 0.33, // default 16:9 zoom
            don_color    : Color::new(0.973, 0.596, 0.651, 1.0),
            kat_color    : Color::new(0.741, 0.698, 0.827, 1.0),

            force_rebuild: false,

            hit_circles : true,

            rebuild_pending : false,
        };
    }
}

pub struct TaikoLayer {
    pub audio : Audio,
    pub clock : SyncClock,

    pub beatmap : Option<Beatmap>,

    pub conveyor : Conveyor,
}

impl TaikoLayer {
    pub fn new(graphics: &Graphics) -> Self {
        return Self {
            audio : Audio::new().unwrap(),
            clock : SyncClock::new(),

            beatmap : None,

            conveyor : Conveyor::new(graphics),
        };
    }
}

impl Layer<&mut TaikoState> for TaikoLayer {
    fn draw<'a: 'b, 'b>(&'a mut self, state: &mut TaikoState, render_pass: &mut wgpu::RenderPass<'b>, graphics: &mut Graphics) {
        let rebuild_instances = state.rebuild_pending || state.force_rebuild;
        if state.rebuild_pending { state.rebuild_pending = false; }
        
        let Some(beatmap) = &self.beatmap else { return };
        let time_ms = self.clock.get_time();
        self.conveyor.draw(rebuild_instances, state, beatmap, time_ms, render_pass, graphics);
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let PhysicalSize::<u32> { width, height } = new_size;
        self.conveyor.scene.projection.resize(width, height);
    }

    fn scale(&mut self, scale: f64) {
        let scale = scale as f32;
        self.conveyor.scene.camera.set_scale(vec3(scale, scale, 1.0));
    }
}

impl TaikoLayer {
    pub fn close_beatmap(&mut self) {
        // Reset clock
        self.clock.set_time(0);
        self.clock.set_paused(true, 0);
        self.clock.set_length(0);

        // Unload audio, reset position
        self.audio.stop();
        self.audio.set_time(Duration::ZERO);

        // Beatmap
        self.beatmap = None;
    }

    // Timeline
    pub fn timeline_move_forward(&mut self, _state: &mut TaikoState, _value: f32) {
    }

    pub fn timeline_move_back(&mut self, _state: &mut TaikoState, _value: f32) {
    }

    // Time
    pub fn toggle_paused(&mut self) {
        let time = self.audio.get_time();
        self.clock.toggle_paused(time.as_millis() as u32);
        self.audio.pause();

        if time.as_millis() as u32 >= self.clock.get_length() {
            self.set_time(0);
        }
    }
    pub fn set_paused(&mut self, value: bool) {
        let time = self.audio.get_time();
        self.clock.set_paused(value, time.as_millis() as u32);
        self.audio.set_paused(value);
    }
    pub fn is_paused(&self) -> bool {
        return self.clock.is_paused();
    }

    pub fn set_time(&mut self, time: u32) {
        let time = Duration::from_millis(time as u64);
        self.clock.set_time(time.as_millis() as u32);
        self.audio.set_time(time);

        // In case of rewind
        self.conveyor.cull_back = 0;
    }
    pub fn get_time(&mut self) -> Time {
        return Time::from_ms(self.clock.get_time());
    }

    pub fn get_length(&self) -> u32 {
        return self.clock.get_length();
    }
}