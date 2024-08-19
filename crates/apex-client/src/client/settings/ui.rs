use std::cell::RefCell;

use egui::Widget as _;
use tap::Pipe as _;

use apex_framework::{
  data::settings::{NumericOpts, StringOpts},
  graphics::color::Color,
};

use crate::client::graphics::{FrameLimiterOptions, PresentModeOptions, RenderingBackend, WgpuBackend};

macro_rules! make_numeric_ui {
  (
    $($ty:ty)+
  ) => {
    paste::paste! {
      $(
        #[allow(dead_code)]
        pub fn [<ui_ $ty:snake>](ui: &mut egui::Ui, value: &$ty, name: &'static str, opts: NumericOpts<$ty>) -> Option<$ty> {
          let mut new_value = None;

          let do_ui = |ui: &mut egui::Ui| {
            ui.label(name);

            let mut value = *value;
            let widget = if opts.slider {
              ui.style_mut().visuals.selection.bg_fill = egui::Color32::from_gray(48);
              ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::from_gray(24);
              ui.style_mut().spacing.slider_width = ui.available_width() - 72.0;

              egui::Slider::new(&mut value, opts.range)
                .step_by(opts.step)
                .trailing_fill(true)
                .clamp_to_range(opts.clamp)
                .pipe(|slider| {
                  let slider = if let Some(decimals) = opts.precision {
                    slider.max_decimals(decimals)
                  } else {
                    slider
                  };

                  let slider = if opts.percentage {
                    slider.custom_formatter(|n, _| format!("{:.0}%", n * 100.0))
                  } else {
                    slider
                  };

                  return slider;
                })
                .ui(ui)
            } else {
              egui::DragValue::new(&mut value)
                .speed(if opts.step == 0.0 { 1.0 } else { opts.step })
                .range(opts.range)
                .clamp_to_range(opts.clamp)
                .max_decimals_opt(opts.precision)
                .ui(ui)
            };

            if widget.changed() {
              new_value = Some(value);
            }
          };

          if opts.inline {
            ui.horizontal(do_ui);
          } else {
            do_ui(ui);
          }

          return new_value;
        }
      )+
    }
  };
}

make_numeric_ui!(
  i8 i16 i32 i64
  u8 u16 u32 u64
  isize usize
  f32 f64
);

pub fn ui_bool(ui: &mut egui::Ui, value: &bool, name: &'static str) -> Option<bool> {
  let mut new_value = None;

  let mut value = *value;
  if egui::Checkbox::new(&mut value, name).ui(ui).changed() {
    new_value = Some(value);
  }

  return new_value;
}

pub fn ui_string(ui: &mut egui::Ui, value: &String, name: &'static str, opts: StringOpts) -> Option<String> {
  let mut new_value = None;

  thread_local! {
    static BUFFER: RefCell<String> = const { RefCell::new(String::new()) };
  }

  let mut do_ui = |ui: &mut egui::Ui| {
    BUFFER.with_borrow_mut(|x| {
      x.clone_from(value);
      if egui::TextEdit::singleline(x)
        .margin(egui::Margin::symmetric(8.0, 6.0))
        .desired_width(ui.available_width() - 40.0)
        .ui(ui)
        .changed()
      {
        new_value = Some(x.clone());
      }
    });
  };

  ui.vertical(|ui| {
    if opts.inline {
      ui.horizontal(|ui| {
        ui.vertical(|ui| {
          ui.add_space(6.0);
          ui.label(name);
        });
        do_ui(ui);
      });
    } else {
      ui.label(name);
      do_ui(ui);
    }
  });

  return new_value;
}

pub fn ui_color(ui: &mut egui::Ui, value: &Color, name: &'static str) -> Option<Color> {
  let mut new_value = None;

  ui.horizontal(|ui| {
    ui.label(name);

    use egui::color_picker::{color_edit_button_rgba, Alpha};
    let (mut r, mut g, mut b, mut a) = value.as_rgba();
    let mut color = egui::Rgba::from_rgba_unmultiplied(r, g, b, a);
    if color_edit_button_rgba(ui, &mut color, Alpha::Opaque).changed() {
      new_value = Some(color.into());
    }

    if { false }
      | egui::DragValue::new(&mut r).speed(0.0025).fixed_decimals(2).range(0.0 ..= 1.0).ui(ui).changed()
      | egui::DragValue::new(&mut g).speed(0.0025).fixed_decimals(2).range(0.0 ..= 1.0).ui(ui).changed()
      | egui::DragValue::new(&mut b).speed(0.0025).fixed_decimals(2).range(0.0 ..= 1.0).ui(ui).changed()
      | egui::DragValue::new(&mut a).speed(0.0025).fixed_decimals(2).range(0.0 ..= 1.0).ui(ui).changed()
    {
      new_value = Some(Color::new(r, g, b, a));
    }
  });

  return new_value;
}

pub fn ui_rendering_backend(
  ui: &mut egui::Ui,
  value: &RenderingBackend,
  name: &'static str,
) -> Option<RenderingBackend> {
  let mut new_value = None;

  let mut selected = *value;
  egui::ComboBox::new("rendering_backend", name)
    .selected_text(format!("{:?}", selected))
    .width(ui.available_width() - 192.0)
    .show_ui(ui, |ui| {
      ui.style_mut().visuals.selection.stroke = egui::Stroke::new(1.5, egui::Color32::from_gray(255));

      if { false }
        || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Auto), "Auto").changed()
        || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Vulkan), "Vulkan").changed()
        || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Metal), "Metal").changed()
        || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Dx12), "DX 12").changed()
        || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Gl), "OpenGL").changed()
        || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::WebGpu), "WebGPU").changed()
      {
        new_value = Some(selected);
      }
    });

  return new_value;
}

pub fn ui_frame_limiter_options(
  ui: &mut egui::Ui,
  value: &FrameLimiterOptions,
  name: &'static str,
) -> Option<FrameLimiterOptions> {
  let mut new_value = None;

  let mut selected = *value;
  egui::ComboBox::new("frame_limiter", name)
    .selected_text(format!("{:?}", selected))
    .width(ui.available_width() - 192.0)
    .show_ui(ui, |ui| {
      ui.style_mut().visuals.selection.stroke = egui::Stroke::new(1.5, egui::Color32::from_gray(255));

      if { false }
        || ui.selectable_value(&mut selected, FrameLimiterOptions::Custom(240), "240 fps").changed()
        || ui.selectable_value(&mut selected, FrameLimiterOptions::Custom(480), "480 fps").changed()
        || ui.selectable_value(&mut selected, FrameLimiterOptions::Custom(960), "960 fps").changed()
        || ui.selectable_value(&mut selected, FrameLimiterOptions::Unlimited, "Unlimited").changed()
        || ui.selectable_value(&mut selected, FrameLimiterOptions::DisplayLink, "Display Link").changed()
      {
        new_value = Some(selected);
      }
    });

  return new_value;
}

pub fn ui_present_mode_options(
  ui: &mut egui::Ui,
  value: &PresentModeOptions,
  name: &'static str,
) -> Option<PresentModeOptions> {
  let mut new_value = None;

  let mut selected = *value;
  egui::ComboBox::new("present_mode", name)
    .selected_text(format!("{:?}", selected))
    .width(ui.available_width() - 192.0)
    .show_ui(ui, |ui| {
      ui.style_mut().visuals.selection.stroke = egui::Stroke::new(1.5, egui::Color32::from_gray(255));

      if { false }
        || ui.selectable_value(&mut selected, PresentModeOptions::VSync, "VSync").changed()
        || ui.selectable_value(&mut selected, PresentModeOptions::Immediate, "Immediate").changed()
      {
        new_value = Some(selected);
      }
    });

  return new_value;
}
