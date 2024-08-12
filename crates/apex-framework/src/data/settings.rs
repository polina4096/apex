use std::ops::RangeInclusive;

use num_traits::Bounded;
use smart_default::SmartDefault;

#[macro_export]
macro_rules! _call_with_custom_attrs {
  (
    $fun:ident<$type:ty>( $( $result:expr, )* );
    #[custom(ui( $($opts:ident = $values:expr),* ))]
    $( #[ $($rest:tt)* ] )*
  ) => {
    {
      paste::paste! {
        let opts = $crate::data::settings::[<Opts $type:camel>] {
          $( $opts: $values, )*
          .. Default::default()
        };
      }

      $crate::_call_with_custom_attrs!(
        $fun<$type>(
          $( $result, )*
          opts,
        );
        $( #[ $($rest)* ] )*
      )
    }
  };

  (
    $fun:ident<$type:ty>( $( $result:expr, )* );
    #[ $attr:meta ]
    $( #[ $($rest:tt)* ] )*
  ) => {
    $crate::_call_with_custom_attrs!(
      $fun<$type>( $( $result, )* );
      $( #[ $($rest)* ] )*
    )
  };

  (
    $fun:ident<$type:ty>( $( $result:expr, )* );
  ) => {
    settings_ui::$fun( $( $result, )* );
  };
}

#[macro_export]
macro_rules! SettingsStruct {
  (
    $( #[ $($attr:tt)* ] )*
    $vis:vis struct $name:ident {
      $( $field_vis:vis $field:ident: $type:ty, )*
    }
  ) => {
    paste::paste! {
      pub trait [<$name Proxy>]: Sized $( + [<$type Proxy>] )* { }
    }

    impl $name {
      pub fn ui(&mut self, body: &mut egui_extras::TableBody, text_height: f32, proxy: &mut impl SettingsProxy) {
        $(
          self.$field.ui(body, text_height, proxy);
        )*
      }
    }
  };
}

#[macro_export]
macro_rules! SettingsGroup {
  (
    $( #[ $($attr:tt)* ] )*
    $vis:vis struct $name:ident {
      $(
        $( #[ $($field_attr:tt)* ] )*
        $field_vis:vis $field:ident: $type:ty,
      )*
    }
  ) => {
    paste::paste! {
      impl $name {
        $(
          pub fn $field(&self) -> $type {
            return self.$field.clone();
          }
          pub fn [<set_ $field>](&mut self, $field: $type, proxy: &mut impl SettingsProxy) {
            self.$field = $field;
            proxy.[<update_ $name:snake _ $field>](&self.$field);
          }

          pub fn [<$field _ref>](&self) -> &$type {
            return &self.$field;
          }
          pub fn [<set_ $field _ref>](&mut self, $field: &$type, proxy: &mut impl SettingsProxy) {
            self.$field.clone_from($field);
            proxy.[<update_ $name:snake _ $field>]($field);
          }
        )*

        pub fn ui(&mut self, body: &mut egui_extras::TableBody, text_height: f32, proxy: &mut impl SettingsProxy) {
          body.row(text_height + 8.0, |mut row| {
            row.col(|ui| {
              let text = egui::RichText::new(stringify!($name)).strong().heading();
              egui::Label::new(text).ui(ui);
            });

            row.col(|_| {});
          });

          $(
            body.row(text_height + 8.0, |mut row| {
              row.col(|ui| {
                ui.label(stringify!($field));
              });

              row.col(|ui| {
                ui.horizontal_centered(|ui| {
                  if egui::Button::new("⟲").frame(false).ui(ui).clicked() {
                    self.[<set_ $field>](Self::default().$field(), proxy);
                  }

                  ui.add_space(4.0);

                  if let Some(new_value) = $crate::_call_with_custom_attrs!(
                    [<ui_ $type:snake>]<$type>(
                      ui,
                      &self.$field(),
                    );
                    $( #[ $($field_attr)* ] )*
                  ) {
                    self.[<set_ $field>](new_value, proxy);
                  }
                });
              });
            });
          )*
        }
      }

      pub trait [<$name Proxy>] {
        $(
          #[allow(clippy::ptr_arg)]
          fn [<update_ $name:snake _ $field>](&mut self, value: &$type) {}
        )*
      }
    }
  };
}

macro_rules! make_numeric_opts {
  (
    $($ty:ty)+
  ) => {
    paste::paste! {
      $(
        pub type [<Opts $ty:camel>] = NumericOpts<$ty>;
      )+
    }
  };
}

make_numeric_opts! {
  i8 i16 i32 i64 i128
  u8 u16 u32 u64 u128
  isize usize
  f32 f64
}

#[derive(SmartDefault)]
pub struct NumericOpts<T: Bounded + Default> {
  #[default(T::min_value() ..= T::max_value())]
  pub range: RangeInclusive<T>,

  #[default = true]
  pub clamp: bool,

  #[default(0.0)]
  pub step: f64,

  #[default(None)]
  pub precision: Option<usize>,

  #[default = true]
  pub slider: bool,
}
