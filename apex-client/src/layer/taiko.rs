use std::{time::Duration, path::Path, collections::HashMap, io::Cursor};

use async_zip::base::read::mem::ZipFileReader;
use cgmath::{vec3, vec2, Vector2};
use wcore::{audio::{Audio, AudioData, Hint}, clock::{SyncClock, Clock}, time::Time, graphics::{context::Graphics, camera::{Projection, Camera}, layer::Layer}, color::Color};
use winit::dpi::PhysicalSize;

use crate::{taiko::{parser::Beatmap, self}, graphics::{taiko::{conveyor::Conveyor, skin::Skin}}};


pub struct TaikoState {
    /* Settings */
    pub scale        : f32,
    pub audio_offset : i64, // ms
    pub hit_position : Vector2<f32>,
    pub zoom         : f32,
    pub don_color    : Color,
    pub kat_color    : Color,

    pub skin         : Skin,

    // Debug
    pub force_rebuild : bool,

    // Temporary
    pub hit_circles : bool,

    /* Internal */
    pub rebuild_pending : bool,
}

impl TaikoState {
    pub fn new(graphics: &Graphics) -> Self {
        return Self {
            scale        : 0.85,
            audio_offset : 65, // osu audio engine (bass) is oof...
            hit_position : vec2(300.0, 300.0),
            zoom         : 0.33, // default 16:9 zoom
            don_color    : Color::new(0.973, 0.596, 0.651, 1.0),
            kat_color    : Color::new(0.741, 0.698, 0.827, 1.0),

            skin : Skin::default(graphics),

            force_rebuild   : false,

            hit_circles     : true,

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

impl<'b> Layer<'b, &'b mut TaikoState> for TaikoLayer {
    fn draw<'a: 'b>(&'a mut self, state: &'b mut TaikoState, render_pass: &mut wgpu::RenderPass<'b>, graphics: &mut Graphics) {
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
    pub fn open_beatmap(&mut self, path: &Path, state: &mut TaikoState) {
        let mut files = HashMap::<String, Vec<u8>>::default();
        pollster::block_on(async {
            let archive = std::fs::read(path).unwrap();
            let archive = ZipFileReader::new(archive).await.unwrap();
            for i in 0 .. archive.file().entries().len() {
                let mut file = archive.entry(i).await.unwrap();
                let mut buffer = vec![];
                file.read_to_end_checked(&mut buffer, archive.file().entries()[i].entry()).await.unwrap();
                files.insert(archive.file().entries()[i].entry().filename().to_owned(), buffer);
            }
        });

        let filename = files.keys().find(|x| x.ends_with(".osu")).unwrap().clone();
        let data = String::from_utf8(files.remove(&filename).unwrap()).unwrap();
        
        // Beatmap
        let beatmap = taiko::parser::try_parse(&data).unwrap();
        
        let audio_filename = beatmap.audio.file_name().unwrap().to_str().unwrap();
        let filename = files.keys().find(|x| *x == audio_filename).unwrap().clone();
        let file = files.remove(&filename).unwrap();
        let audio_file = Cursor::new(file);
        self.beatmap = Some(beatmap);

        let audio_data = AudioData::new(
            Box::new(audio_file),
            Hint::new().with_extension("mp3")
        ).unwrap();

        self.audio.play(&audio_data).unwrap();

        // Update clock data
        self.clock.set_time(0);
        self.clock.set_paused(true, 0);
        self.clock.set_length(self.audio.length().as_millis() as u32);

        // Build instances
        state.rebuild_pending = true;
    }

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