use std::{collections::HashMap, io::Cursor};

use async_zip::base::read::mem::ZipFileReader;
use egui_file::FileDialog;
use wcore::{graphics::{gui::{view::View, window::Window}, context::Graphics}, audio::{AudioData, Hint}, clock::Clock};

use crate::{state::AppState, taiko::parser, layer::Layers};

pub struct FileDialogWindow {
    open   : bool,
    dialog : FileDialog,
}

impl FileDialogWindow {
    pub fn new() -> Self {
        return Self {
            open   : false,
            dialog : FileDialog::open_file(None),
        };
    }
}

impl Window<()> for FileDialogWindow {
    type Title = &'static str;
    fn title() -> Self::Title {
        return "Choose a file";
    }

    fn build<'b>(window: egui::Window<'b>, _ctx: &'_ egui::Context) -> egui::Window<'b> {
        window
            .collapsible(false)
            .resizable(true)
            .default_pos(egui::pos2(8.0, 32.0))
            .title_bar(true)
    }

    fn set_visible(&mut self, value: bool) { self.open = value; if value { self.dialog.open(); } }
    fn get_visible(&self) -> bool { return self.open; }

    #[allow(unused_variables)]
    fn show(&mut self, state: (), view: &wgpu::TextureView, graphics: &mut Graphics, ui: &mut egui::Ui) { }
}

// Hand-rolling a window view impl 
impl View<(&mut AppState, &mut Layers)> for FileDialogWindow {
    #[allow(unused_variables)]
    fn show(&mut self, (state, layers): (&mut AppState, &mut Layers), view: &wgpu::TextureView, graphics: &mut Graphics, ctx: &egui::Context) {
        if self.dialog.show(ctx).selected() {
            if let Some(path) = self.dialog.path() {
                let mut files = HashMap::<String, Vec<u8>>::default();
                pollster::block_on(async {
                    let archive = std::fs::read(&path).unwrap();
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
                let beatmap = parser::try_parse(&data).unwrap();
                
                let audio_filename = beatmap.audio.file_name().unwrap().to_str().unwrap();
                let filename = files.keys().find(|x| *x == audio_filename).unwrap().clone();
                let file = files.remove(&filename).unwrap();
                let audio_file = Cursor::new(file);
                layers.taiko.beatmap = Some(beatmap);

                let audio_data = AudioData::new(
                    Box::new(audio_file),
                    Hint::new().with_extension("mp3")
                ).unwrap();

                layers.taiko.audio.play(&audio_data).unwrap();

                // Update clock data
                layers.taiko.clock.set_time(0);
                layers.taiko.clock.set_paused(true, 0);
                layers.taiko.clock.set_length(layers.taiko.audio.length().as_millis() as u32);

                // Build instances
                state.taiko.rebuild_pending = true;
            }
        }
    }
}