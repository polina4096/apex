use std::{time::Duration, path::{Path, PathBuf}, collections::HashMap, io::Cursor};

use async_zip::base::read::mem::ZipFileReader;
use cgmath::{vec3, vec2, Vector2, Zero};
use wcore::{audio::{Audio, AudioData, Hint}, clock::{SyncClock, Clock}, time::Time, graphics::{context::Graphics, camera::{Projection, Camera}, layer::Layer}, color::Color};
use winit::dpi::PhysicalSize;
use xxhash_rust::xxh3;

use crate::{taiko::{parser::Beatmap, self}, graphics::{taiko::{conveyor::Conveyor, skin::Skin}}};


pub struct TaikoState {
    /* Settings */
    pub scale        : f64,
    pub audio_offset : f64, // ms
    pub hit_position : Vector2<f64>,
    pub zoom         : f64,
    pub don_color    : Color,
    pub kat_color    : Color,

    pub skin         : Skin,

    // Debug
    pub force_rebuild : bool,

    // Temporary
    pub hide_circles : bool,

    /* Internal */
    pub rebuild_pending : bool,
}

impl TaikoState {
    pub fn new(graphics: &Graphics) -> Self {
        return Self {
            scale        : 0.85,
            audio_offset : 0.0,
            hit_position : vec2(300.0, 300.0),
            zoom         : 0.235, // default 16:9 zoom
            don_color    : Color::new(0.973, 0.596, 0.651, 1.0),
            kat_color    : Color::new(0.741, 0.698, 0.827, 1.0),

            skin : Skin::default(graphics),

            force_rebuild   : false,

            hide_circles    : false,

            rebuild_pending : false,
        };
    }
}

pub struct TaikoLayer {
    pub audio  : Audio,
    pub clock  : SyncClock,
    audio_hash : u128,

    pub beatmap      : Option<Beatmap>,
    pub beatmap_path : Option<PathBuf>,

    pub conveyor : Conveyor,

    pub snapping : u8,
}

impl TaikoLayer {
    pub fn new(graphics: &Graphics) -> Self {
        return Self {
            audio : Audio::new().unwrap(),
            clock : SyncClock::new(),
            audio_hash : Default::default(),

            beatmap      : None,
            beatmap_path : None,

            conveyor : Conveyor::new(graphics),
            
            snapping : 4,
        };
    }
}

impl<'b> Layer<'b, &'b mut TaikoState> for TaikoLayer {
    fn draw<'a: 'b>(&'a mut self, state: &'b mut TaikoState, render_pass: &mut wgpu::RenderPass<'b>, graphics: &mut Graphics) {
        let rebuild_instances = state.rebuild_pending || state.force_rebuild;
        if state.rebuild_pending { state.rebuild_pending = false; }
        
        let time = self.clock.get_time();
        let Some(beatmap) = &self.beatmap else { return };
        self.conveyor.draw(rebuild_instances, state, beatmap, time, render_pass, graphics);
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
        let Some(Some(ext)) = path.extension().map(|x| x.to_str()) else { return };
        match ext {
            "osu" => self.open_beatmap_osu(path, state),
            "osz" => self.open_beatmap_osz(path, state),

            _ => {}
        }
    }
    pub fn open_beatmap_osu(&mut self, path: &Path, state: &mut TaikoState) {
        let Ok(data) = std::fs::read_to_string(path) else { return };

        // Beatmap
        let beatmap = taiko::parser::try_parse(&data).unwrap();

        let audio_filename = beatmap.audio.file_name().unwrap().to_str().unwrap();
        let audio_path = path.parent().unwrap().join(audio_filename);
        let audio_data = std::fs::read(audio_path).unwrap();

        self.load_beatmap(beatmap, audio_data, state);

        self.beatmap_path = Some(path.to_owned());
    }
    pub fn open_beatmap_osz(&mut self, path: &Path, state: &mut TaikoState) {
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
        let audio_data = files.remove(audio_filename).unwrap();

        self.load_beatmap(beatmap, audio_data, state);
        
        self.beatmap_path = None;
    }

    pub fn load_beatmap(&mut self, beatmap: Beatmap, audio_data: Vec<u8>, state: &mut TaikoState) {
        self.beatmap = Some(beatmap);

        // Audio
        let hash = xxh3::xxh3_128(&audio_data);
        if hash != self.audio_hash {
            self.audio_hash = hash;

            let audio_data = AudioData::new(
                Box::new(Cursor::new(audio_data)),
                Hint::new().with_extension("mp3")
            ).unwrap();
    
            self.audio.play(&audio_data).unwrap();
        
            self.audio.set_paused(true);
            self.audio.set_time(Duration::ZERO);
    
            // Update clock data
            self.clock.set_time(Time::zero());
            self.clock.set_paused(true, Time::zero());
            self.clock.set_length(self.audio.length().into());
        }

        // Build instances
        state.rebuild_pending = true;

        // Reset culling
        self.conveyor.cull_back = 0;
    }
    pub fn unload_beatmap(&mut self) {
        // Reset clock
        self.clock.set_time(Time::zero());
        self.clock.set_paused(true, Time::zero());
        self.clock.set_length(Time::zero());

        // Unload audio, reset position
        self.audio.stop();
        self.audio.set_time(Duration::ZERO);

        // Beatmap
        self.beatmap = None;
        self.beatmap_path = None;
        self.audio_hash = Default::default();
    }

    // Timeline
    pub fn timeline_move(&mut self, _state: &mut TaikoState, value: f32, snapping: u8) {
        if value.is_zero() { return }
        
        let time = self.get_time();

        // Find current timing point
        let Some(beatmap) = &self.beatmap else { return };

        let mut idx_t = beatmap.timing.len() - 1;
        while beatmap.timing[idx_t].time > time && idx_t != 0 { idx_t -= 1; }
        let timing_point = &beatmap.timing[idx_t];

        // Beat length divided by current beat snapping
        let snap_length = Time::from_seconds(60.0 / timing_point.bpm / snapping as f64);

        if !self.is_paused() {
            if value > 0.0
                 { self.set_time(time + snap_length); }
            else { self.set_time(time - snap_length); }
            return;
        }
        
        // Amount of snaps from the timing point
        let snap_count = ((time - timing_point.time) / snap_length).to_seconds();

        const SNAP_DISTANCE: f64 = 0.005;

        if value > 0.0 {
            let new_time = Time::from_seconds(
                snap_count.ceil()
              * snap_length.to_seconds()
              + timing_point.time.to_seconds()
            );

            if new_time - time < Time::from_seconds(SNAP_DISTANCE)
                 { self.set_time(time + snap_length); }
            else { self.set_time(new_time); }
        } else {
            let new_time = Time::from_seconds(
                snap_count.floor()
              * snap_length.to_seconds()
              + timing_point.time.to_seconds()
            );

            if time - new_time < Time::from_seconds(SNAP_DISTANCE)
                 { self.set_time(time - snap_length); }
            else { self.set_time(new_time); }
        }
    }

    // Time
    pub fn toggle_paused(&mut self) {
        let time = self.audio.get_time();
        self.clock.toggle_paused(time.into());
        self.audio.pause();

        if time >= self.clock.get_length().into() {
            self.set_time(Time::zero());
        }
    }
    pub fn set_paused(&mut self, value: bool) {
        let time = self.audio.get_time();
        self.clock.set_paused(value, time.into());
        self.audio.set_paused(value);
    }
    pub fn is_paused(&self) -> bool {
        return self.clock.is_paused();
    }

    pub fn set_time(&mut self, mut time: Time) {
        if time < Time::zero() {
            time = Time::zero()
        } else {
            let length = self.get_length();
            if time > length {
                time = length
            }
        }

        self.clock.set_time(time);
        self.audio.set_time(time.into());

        // In case of rewind
        self.conveyor.cull_back = 0;
    }
    pub fn get_time(&mut self) -> Time {
        return self.clock.get_time();
    }

    pub fn get_length(&self) -> Time {
        return self.clock.get_length();
    }
}