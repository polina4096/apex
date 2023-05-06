use std::{time::Duration, path::{Path, PathBuf}, collections::HashMap, io::Cursor};

use async_zip::base::read::mem::ZipFileReader;
use cgmath::{vec3, vec2, Vector2, Zero};
use log::warn;
use wcore::{audio::{Audio, AudioData, Hint}, clock::{SyncClock, Clock}, time::Time, graphics::{context::Graphics, camera::{Projection, Camera}, layer::Layer}, color::Color, binds::{KeyCombination, Keybinds, KeyCode, Bind}};
use winit::{dpi::PhysicalSize, event::{VirtualKeyCode, ModifiersState}};
use xxhash_rust::xxh3;

use crate::{taiko::{parser::Beatmap, self, taiko_circle::{TaikoColor, TaikoCircle}}, graphics::{taiko::{conveyor::Conveyor, skin::Skin}}};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaikoKeybinds {
    TogglePlayback,

    TimelineMoveForward,
    TimelineMoveBack,
    TimelineJumpBeatmapStart,
    TimelineJumpBeatmapEnd,

    ObjectPlaceKat,
    ObjectPlaceDon,
    ObjectToggleSize,
    ObjectRemove,
}

pub struct TaikoState {
    pub keybinds : Keybinds<TaikoKeybinds>,
    
    /* Settings */
    pub scale        : f64,
    pub audio_offset : f64, // ms
    pub hit_position : Vector2<f64>,
    pub zoom         : f64,
    pub don_color    : Color,
    pub kat_color    : Color,

    // Debug
    pub force_rebuild : bool,

    // Temporary
    pub hide_circles : bool,
}

impl TaikoState {
    pub fn new() -> Self {
        let mut keybinds = Keybinds::default();

        keybinds.add(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::Space), modifiers: ModifiersState::empty() },
            Bind { id: TaikoKeybinds::TogglePlayback, name: String::from("play/pause"), description: String::from("starts or stops playback") }
        );

        keybinds.add(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::Right), modifiers: ModifiersState::empty() },
            Bind { id: TaikoKeybinds::TimelineMoveForward, name: String::from("Timeline forward"), description: String::from("Move 1/n of a beat forward on a timeline in the song") }
        );

        keybinds.add(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::Left), modifiers: ModifiersState::empty() },
            Bind { id: TaikoKeybinds::TimelineMoveBack, name: String::from("Timeline back"), description: String::from("Move 1/n of a beat back on a timeline in the song") }
        );

        keybinds.add(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::Z), modifiers: ModifiersState::empty() },
            Bind { id: TaikoKeybinds::TimelineJumpBeatmapStart, name: String::from("Beatmap start"), description: String::from("Jumps to the start of the beatmap") }
        );

        keybinds.add(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::V), modifiers: ModifiersState::empty() },
            Bind { id: TaikoKeybinds::TimelineJumpBeatmapEnd, name: String::from("Beatmap end"), description: String::from("Jumps to the end of the beatmap") }
        );

        keybinds.add(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::A), modifiers: ModifiersState::empty() },
            Bind { id: TaikoKeybinds::ObjectPlaceKat, name: String::from("Place kat"), description: String::from("Places a kat at the current position") }
        );

        keybinds.add(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::S), modifiers: ModifiersState::empty() },
            Bind { id: TaikoKeybinds::ObjectPlaceDon, name: String::from("Place don"), description: String::from("Places a don at the current position") }
        );

        keybinds.add(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::D), modifiers: ModifiersState::empty() },
            Bind { id: TaikoKeybinds::ObjectRemove, name: String::from("Remove circle"), description: String::from("Removes object the current position") }
        );

        return Self {
            keybinds,
            
            scale        : 0.85,
            audio_offset : 0.0,
            hit_position : vec2(300.0, 300.0),
            zoom         : 0.235, // default 16:9 zoom
            don_color    : Color::new(0.973, 0.596, 0.651, 1.0),
            kat_color    : Color::new(0.741, 0.698, 0.827, 1.0),

            force_rebuild   : false,

            hide_circles    : false,
        };
    }
}

pub struct TaikoLayer {
    audio_hash          : u128,
    pub audio           : Audio,
    pub clock           : SyncClock,
    pub beatmap         : Option<Beatmap>,
    pub beatmap_path    : Option<PathBuf>,
    
    pub conveyor        : Conveyor,
    
    pub skin            : Skin,
    pub is_kat          : bool,
    pub is_big          : bool,
    pub snapping        : u8,

    pub rebuild_pending : bool,
}

impl TaikoLayer {
    pub fn new(graphics: &Graphics) -> Self {
        return Self {
            audio_hash      : Default::default(),
            audio           : Audio::new().unwrap(),
            clock           : SyncClock::new(),
            beatmap         : None,
            beatmap_path    : None,
            
            conveyor        : Conveyor::new(graphics),
            
            skin            : Skin::default(graphics),
            is_kat          : false,
            is_big          : false,
            snapping        : 4,

            rebuild_pending : false,
        };
    }
}

impl<'b> Layer<'b, &'b mut TaikoState> for TaikoLayer {
    fn draw<'a: 'b>(&'a mut self, state: &'b mut TaikoState, render_pass: &mut wgpu::RenderPass<'b>, graphics: &mut Graphics) {
        let rebuild_instances = self.rebuild_pending || state.force_rebuild;
        if self.rebuild_pending { self.rebuild_pending = false; }
        
        let time = self.clock.get_time();
        let Some(beatmap) = &self.beatmap else { return };
        self.conveyor.draw(rebuild_instances, state, beatmap, time, &self.skin, render_pass, graphics);
    }

    fn action(&mut self, keys: KeyCombination, state: &'b mut TaikoState) -> bool {
        if let Some(keybind) = state.keybinds.get(&keys) {
            match keybind.id {
                TaikoKeybinds::TogglePlayback => {
                    self.toggle_paused();
                }

                TaikoKeybinds::TimelineMoveForward      => self.timeline_move(state,  1.0, self.snapping),
                TaikoKeybinds::TimelineMoveBack         => self.timeline_move(state, -1.0, self.snapping),
                TaikoKeybinds::TimelineJumpBeatmapStart => self.timeline_jump_beatmap_start(),
                TaikoKeybinds::TimelineJumpBeatmapEnd   => self.timeline_jump_beatmap_end(),
                
                TaikoKeybinds::ObjectPlaceKat => self.object_place(TaikoColor::KAT),
                TaikoKeybinds::ObjectPlaceDon => self.object_place(TaikoColor::DON),
                TaikoKeybinds::ObjectToggleSize => todo!(),
                TaikoKeybinds::ObjectRemove => self.object_remove(),
            }
        }
        
        return false;
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
    pub fn open_beatmap(&mut self, path: &Path) {
        let Some(Some(ext)) = path.extension().map(|x| x.to_str()) else { return };
        match ext {
            "osu" => self.open_beatmap_osu(path),
            "osz" => self.open_beatmap_osz(path),

            _ => {}
        }
    }
    pub fn open_beatmap_osu(&mut self, path: &Path) {
        let Ok(data) = std::fs::read_to_string(path) else { return };

        // Beatmap
        let beatmap = taiko::parser::try_parse(&data).unwrap();

        let audio_filename = beatmap.audio.file_name().unwrap().to_str().unwrap();
        let audio_path = path.parent().unwrap().join(audio_filename);
        let audio_data = std::fs::read(audio_path).unwrap();

        self.load_beatmap(beatmap, audio_data);

        self.beatmap_path = Some(path.to_owned());
    }
    pub fn open_beatmap_osz(&mut self, path: &Path) {
        let mut files = HashMap::<String, Vec<u8>>::default();
        pollster::block_on(async {
            let archive = std::fs::read(path).unwrap();
            let archive = ZipFileReader::new(archive).await.unwrap();
            for i in 0 .. archive.file().entries().len() {
                let mut file = archive.reader_with_entry(i).await.unwrap();
                let mut buffer = vec![];
                file.read_to_end_checked(&mut buffer).await.unwrap();
                files.insert(archive.file().entries()[i].entry().filename().as_str().unwrap().to_owned(), buffer);
            }
        });

        let filename = files.keys().find(|x| x.ends_with(".osu")).unwrap().clone();
        let data = String::from_utf8(files.remove(&filename).unwrap()).unwrap();
        
        // Beatmap
        let beatmap = taiko::parser::try_parse(&data).unwrap();

        let audio_filename = beatmap.audio.file_name().unwrap().to_str().unwrap();
        let audio_data = files.remove(audio_filename).unwrap();

        self.load_beatmap(beatmap, audio_data);
        
        self.beatmap_path = None;
    }

    pub fn load_beatmap(&mut self, beatmap: Beatmap, audio_data: Vec<u8>) {
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
        self.rebuild_pending = true;

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

    // Object
    pub fn object_place(&mut self, color: TaikoColor) {
        let time = self.get_time();
        let Some(beatmap) = &mut self.beatmap else { return };

        let mut idx_t = beatmap.timing.len() - 1;
        while beatmap.timing[idx_t].time > time && idx_t != 0 { idx_t -= 1; }
        let timing_point = &beatmap.timing[idx_t];

        // Beat length divided by current beat snapping
        let snap_length = Time::from_seconds(60.0 / timing_point.bpm / self.snapping as f64);
        let snap_count = ((time - timing_point.time) / snap_length).to_seconds();

        let closest_time = Time::from_seconds(
            snap_count.round()
          * snap_length.to_seconds()
          + timing_point.time.to_seconds()
        );

        let snap_distance = Time::from_ms(5.0 / 2.0);
        if let Some(obj) = beatmap.objects.iter_mut().find(|x|
                x.time < (closest_time + snap_distance) &&
                x.time > (closest_time - snap_distance)) {
            obj.color = color;
        } else {
            beatmap.objects.push(TaikoCircle {
                time  : closest_time,
                big   : false,
                color : color,
            })
        }

        // TODO: this can be further optimized by inserting the circle into the correct position
        beatmap.objects.sort_by(|a, b| a.time
            .partial_cmp(&b.time)
            .unwrap_or_else(|| {
                warn!("Failed to compare object times, ");
                std::cmp::Ordering::Equal }));

        self.rebuild_pending = true;
    }

    pub fn object_remove(&mut self) {
        let time = self.get_time();
        let Some(beatmap) = &mut self.beatmap else { return };

        let mut idx_t = beatmap.timing.len() - 1;
        while beatmap.timing[idx_t].time > time && idx_t != 0 { idx_t -= 1; }
        let timing_point = &beatmap.timing[idx_t];

        // Beat length divided by current beat snapping
        let snap_length = Time::from_seconds(60.0 / timing_point.bpm / self.snapping as f64);
        let snap_count = ((time - timing_point.time) / snap_length).to_seconds();

        let closest_time = Time::from_seconds(
            snap_count.round()
          * snap_length.to_seconds()
          + timing_point.time.to_seconds()
        );

        let snap_distance = Time::from_ms(5.0 / 2.0);
        if let Some(idx) = beatmap.objects.iter().position(|x|
                x.time < (closest_time + snap_distance) &&
                x.time > (closest_time - snap_distance)) {
            beatmap.objects.remove(idx);
        }

        self.rebuild_pending = true;
    }

    // Timeline
    pub fn timeline_jump_beatmap_start(&mut self) {
        let Some(beatmap) = &self.beatmap else { return };
        if let Some(obj) = beatmap.objects.first() {
            let time = self.clock.get_time();
            if obj.time != time {
                self.set_time(obj.time);
                return;
            }
        }

        self.set_time(Time::zero());
    }
    pub fn timeline_jump_beatmap_end(&mut self) {
        let Some(beatmap) = &self.beatmap else { return };
        if let Some(obj) = beatmap.objects.last() {
            let time = self.clock.get_time();
            if obj.time != time {
                self.set_time(obj.time);
                return;
            }
        }

        self.set_time(self.get_length());
    }

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
                 { self.set_time(time + snap_length * 6.0); }
            else { self.set_time(time - snap_length * 6.0); }
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