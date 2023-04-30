use log::error;
use wcore::{graphics::{context::Graphics, gui::{view::View, window::Window as _}, layer::Layer}, egui::Egui, binds::KeyCombination};
use winit::{window::Window, event::{WindowEvent, ElementState, MouseScrollDelta, VirtualKeyCode}, event_loop::{EventLoop, EventLoopProxy}};

use crate::{config::Config, view::{window::{timeline::TimelineWindow, file_dialog::FileDialogWindow, controls::ControlsWindow}, menu::MenuView, sidebar::SidebarView}, state::{AppState, AppKeybinds, AppEvents}, graphics::util::new_graphics, layer::{taiko::TaikoLayer, Layers}, input::Input};

pub struct App {
    // events
    pub proxy : EventLoopProxy<AppEvents>,

    // graphics
    pub window   : Window,
    pub graphics : Graphics,

    // egui
    pub egui : Egui,
    
    pub menu        : MenuView,
    pub sidebar     : SidebarView,
    pub timeline    : TimelineWindow,

    pub file_dialog : FileDialogWindow,
    pub controls    : ControlsWindow,

    // app
    pub state   : AppState,
    pub layers  : Layers,
}

impl App {
    pub async fn new(window: Window, event_loop: &EventLoop<AppEvents>, proxy: EventLoopProxy<AppEvents>, config: &Config) -> Self {
        let graphics = new_graphics(&window, config).await;
        let (input, tx) = Input::new();

        // egui
        let scale = graphics.scale;
        let inner_size = graphics.size;
        let egui = Egui::new(event_loop, &graphics, inner_size.width, inner_size.height, scale);

        // views
        let menu = MenuView::new();
        let sidebar = SidebarView::new();
        let timeline = TimelineWindow::new();
        let file_dialog = FileDialogWindow::new();
        let controls = ControlsWindow::new(tx);

        // state
        let state = AppState::new(&graphics, input);

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
            controls,

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
            View::show(&mut self.menu,        (&mut self.state, &mut self.layers, &self.proxy), &view, graphics, ctx);
            View::show(&mut self.timeline,    (&mut self.state, &mut self.layers), &view, graphics, ctx);
            View::show(&mut self.file_dialog, (&mut self.state, &mut self.layers), &view, graphics, ctx);
            View::show(&mut self.controls,    (&mut self.state, &mut self.layers), &view, graphics, ctx);
            View::show(&mut self.sidebar,     (&mut self.state, &mut self.layers), &view, graphics, ctx);
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

                    self.state.input.key = combination.key;
                    self.state.input.modifiers = mods;

                    #[allow(clippy::collapsible_if)]
                    'a: { if self.state.input.requests_input {
                        if key == VirtualKeyCode::Escape {
                            self.state.input.requests_input = false;
                            break 'a;
                        }

                        if key != VirtualKeyCode::LControl && key != VirtualKeyCode::RControl
                        && key != VirtualKeyCode::LShift   && key != VirtualKeyCode::RShift
                        && key != VirtualKeyCode::LAlt     && key != VirtualKeyCode::RAlt
                        && key != VirtualKeyCode::LWin     && key != VirtualKeyCode::RWin {
                            self.state.input.requests_input = false;
                            self.state.input.input_sender.send(())
                                .unwrap_or_else(|err| error!("{}", err));

                            return true;
                        }
                    } }

                    if let Some(keybind) = self.state.keybinds.get(&combination) {
                        match keybind.id {
                            AppKeybinds::TogglePlayback => {
                                self.layers.taiko.toggle_paused();
                            }

                            AppKeybinds::ToggleSidebar => {
                                self.state.sidebar.shown = !self.state.sidebar.shown;
                            }

                            AppKeybinds::TimelineMoveForward => self.layers.taiko.timeline_move(&mut self.state.taiko,  1.0, self.layers.taiko.snapping),
                            AppKeybinds::TimelineMoveBack    => self.layers.taiko.timeline_move(&mut self.state.taiko, -1.0, self.layers.taiko.snapping),
                        }
                    }
                }
            }

            WindowEvent::MouseWheel { delta, .. } => 'a: {
                let MouseScrollDelta::LineDelta(_, y) = *delta else { break 'a };
                self.layers.taiko.timeline_move(&mut self.state.taiko, -y, self.layers.taiko.snapping);
            }

            WindowEvent::DroppedFile(path) => {
                self.layers.taiko.open_beatmap(path.as_path(), &mut self.state.taiko);
            }

            _ => {}
        }

        return self.egui.winit_state.on_event(&self.egui.context, event).consumed;
    }

    pub fn event(&mut self, event: AppEvents) {
        match event {
            AppEvents::OpenFilePicker => self.file_dialog.set_visible(true),
            AppEvents::OpenControls => self.controls.set_visible(true),
        }
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