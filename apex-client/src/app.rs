use wcore::{graphics::{context::Graphics, gui::view::View, layer::Layer}, egui::Egui, binds::{KeyCombination, KeyCode, Actions, Action}};
use winit::{window::Window, event::{WindowEvent, VirtualKeyCode, ElementState, ModifiersState}, event_loop::{EventLoop, EventLoopProxy}};

use crate::{config::Config, view::{window::{timeline::TimelineWindow, file_dialog::FileDialogWindow}, menu::MenuView, sidebar::SidebarView}, state::AppState, graphics::util::new_graphics, layer::{taiko::TaikoLayer, Layers}};

pub struct App<T: 'static> {
    // events
    pub proxy : EventLoopProxy<T>,

    // graphics
    pub window   : Window,
    pub graphics : Graphics,

    // egui
    pub egui : Egui,
    
    pub menu        : MenuView,
    pub sidebar     : SidebarView,
    pub timeline    : TimelineWindow,

    pub file_dialog : FileDialogWindow,

    // app
    pub actions : Actions<AppActions>,
    pub state   : AppState,
    pub layers  : Layers,
}

pub enum AppEvents {
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppActions {
    TogglePlayback,
    ToggleSidebar,

    TimelineMoveForward,
    TimelineMoveBack,
}

impl<T> App<T> {
    pub async fn new(window: Window, event_loop: &EventLoop<T>, proxy: EventLoopProxy<T>, config: &Config) -> Self {
        let graphics = new_graphics(&window, config).await;
        let mut actions = Actions::default();

        actions.insert(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::Space), modifier: ModifiersState::empty() },
            Action { id: AppActions::TogglePlayback, name: String::from("play/pause"), description: String::from("starts or stops playback") }
        );

        actions.insert(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::O), modifier: ModifiersState::CTRL },
            Action { id: AppActions::ToggleSidebar, name: String::from("toggle sidebar"), description: String::from("shows or hides the sidebar") }
        );

        actions.insert(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::Right), modifier: ModifiersState::empty() },
            Action { id: AppActions::TimelineMoveForward, name: String::from("Timeline forward"), description: String::from("Move 1/n of a beat forward on a timeline in the song") }
        );

        actions.insert(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::Left), modifier: ModifiersState::empty() },
            Action { id: AppActions::TimelineMoveBack, name: String::from("Timeline back"), description: String::from("Move 1/n of a beat back on a timeline in the song") }
        );
        
        // egui
        let scale = graphics.scale;
        let inner_size = graphics.size;
        let egui = Egui::new(event_loop, &graphics, inner_size.width, inner_size.height, scale);

        // views
        let menu = MenuView::new();
        let sidebar = SidebarView::new();
        let timeline = TimelineWindow::new();
        let file_dialog = FileDialogWindow::new();

        // state
        let state = AppState::new(&graphics);

        // layers
        let layers = Layers {
            taiko: TaikoLayer::new(&graphics),
        };

        return Self {
            proxy,

            window,
            graphics,

            egui,

            menu,
            sidebar,
            timeline,
    
            file_dialog,

            actions,
            state,
            layers,
        };
    }

    pub fn update(&mut self) {
        /* ... */
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.graphics.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        let (clipped_primitives, commands) = self.egui.prepare(&self.window, &mut self.graphics, &mut encoder, |graphics, ctx| {
            View::show(&mut self.menu,        (&mut self.state, &mut self.layers, &mut self.file_dialog), &view, graphics, ctx);
            View::show(&mut self.timeline,    (&mut self.state, &mut self.layers), &view, graphics, ctx);
            View::show(&mut self.sidebar,     (&mut self.state, &mut self.layers), &view, graphics, ctx);
            View::show(&mut self.file_dialog, (&mut self.state, &mut self.layers), &view, graphics, ctx);
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.005,
                            g: 0.005,
                            b: 0.005,
                            a: 1.000,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            Layer::draw(&mut self.layers.taiko, &mut self.state.taiko, &mut render_pass, &mut self.graphics);
            self.egui.render(&self.graphics, &mut render_pass, &clipped_primitives, commands);
        }
    
        // submit will accept anything that implements IntoIter
        self.graphics.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    
        return Ok(());
    }

    #[allow(clippy::single_match, deprecated)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode && input.state == ElementState::Pressed {
                    let mods = input.modifiers;
                    let combination = KeyCombination::from((key, mods));
                    if let Some(action) = self.actions.get_mut(&combination) {
                        match action.id {
                            AppActions::TogglePlayback => {
                                self.layers.taiko.toggle_paused();
                            }

                            AppActions::ToggleSidebar => {
                                self.state.sidebar.shown = !self.state.sidebar.shown;
                            }

                            AppActions::TimelineMoveForward => {
                                self.layers.taiko.timeline_move_forward(&mut self.state.taiko, 1.0);
                            }

                            AppActions::TimelineMoveBack => {
                                self.layers.taiko.timeline_move_back(&mut self.state.taiko, 1.0);
                            }
                        }
                    }
                }
            }

            WindowEvent::DroppedFile(path) => {
                self.layers.taiko.open_beatmap(path.as_path(), &mut self.state.taiko);
            }

            _ => {}
        }

        return self.egui.winit_state.on_event(&self.egui.context, event).consumed;
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.graphics.size = new_size;
            self.graphics.config.width = new_size.width;
            self.graphics.config.height = new_size.height;
            self.egui.screen_desc.size_in_pixels = [new_size.width, new_size.height];
            self.graphics.surface.configure(&self.graphics.device, &self.graphics.config);
        }
        
        self.layers.taiko.resize(new_size);
    }

    pub fn scale(&mut self, scale: f64) {
        self.graphics.scale = scale;
        self.egui.scale(scale);
        self.layers.taiko.scale(scale);
    } 

    pub fn get_window(&self) -> &Window {
        return &self.window;
    }
}