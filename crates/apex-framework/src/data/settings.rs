use std::ops::RangeInclusive;

use num_traits::Bounded;
use smart_default::SmartDefault;

#[macro_export]
macro_rules! _call_with_custom_attrs {
  (
    $fun:ident<$type:ty>( $( $result:expr, )* );
    #[custom(ui( name = $custom_name:expr ))]
    $( #[ $($rest:tt)* ] )*
  ) => {
    {
      $crate::_call_with_custom_attrs!(
        $fun<$type>(
          $( $result, )*
          $custom_name,
        );
        $( #[ $($rest)* ] )*
      )
    }
  };

  (
    $fun:ident<$type:ty>( $( $result:expr, )* );
    #[custom(ui( name = $custom_name:expr, $($opts:ident = $values:expr),* ))]
    $( #[ $($rest:tt)* ] )*
  ) => {
    {
      paste::paste! {
        #[allow(clippy::needless_update)]
        let opts = $crate::data::settings::[<$type:camel Opts>] {
          $( $opts: $values, )*
          .. Default::default()
        };
      }

      $crate::_call_with_custom_attrs!(
        $fun<$type>(
          $( $result, )*
          $custom_name,
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
macro_rules! _def_with_custom_attrs {
  ( $name:ident, $size:expr, $separator:expr, $( $field:ident, #[custom(ui(name = $custom_name:expr))] $( #[ $($rest:tt)* ] )* )* ) => {
    $crate::_def_with_custom_attrs!( $name, $size, $separator, impl $( $field, $custom_name, )* );
  };

  ( $name:ident, $size:expr, $separator:expr, impl $( $field:ident, $custom_name:expr, )* ) => {
    impl $name {
      pub fn ui(&mut self, ui: &mut egui::Ui, proxy: &mut impl SettingsProxy) {
        $(
          ui.horizontal(|ui| {
            ui.label(egui::RichText::new($custom_name).size($size).strong());
            if $separator {
              ui.add_space(-10.0);
              ui.add(egui::Separator::default().horizontal().shrink(24.0).spacing(0.0));
            }
          });

          ui.add_space(6.0);

          self.$field.ui(ui, proxy);
        )*
      }
    }
  };
}

#[macro_export]
macro_rules! SettingsStruct {
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
      pub trait [<$name Proxy>]: Sized $( + [<$type Proxy>] )* { }
    }

    $crate::_def_with_custom_attrs!(
      $name,
      24.0,
      true,
      $(
        $field,
        $( #[ $($field_attr)* ] )*
      )*
    );
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
      pub trait [<$name Proxy>]: Sized $( + [<$type Proxy>] )* { }
    }

    $crate::_def_with_custom_attrs!(
      $name,
      18.0,
      false,
      $(
        $field,
        $( #[ $($field_attr)* ] )*
      )*
    );
  };
}

#[macro_export]
macro_rules! SettingsSubgroup {
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
            proxy.[<update_ $field>](&self.$field);
          }

          pub fn [<$field _ref>](&self) -> &$type {
            return &self.$field;
          }
          pub fn [<set_ $field _ref>](&mut self, $field: &$type, proxy: &mut impl SettingsProxy) {
            self.$field.clone_from($field);
            proxy.[<update_ $field>]($field);
          }
        )*

        pub fn ui(&mut self, ui: &mut egui::Ui, proxy: &mut impl SettingsProxy) {
          ui.add_space(4.0);

          $(
            ui.horizontal(|ui| {
              // let button = egui::Button::new("⟲").min_size(egui::Vec2::splat(12.0)).frame(false).ui(ui);

              // if button.clicked() {
              //   self.[<set_ $field>](Self::default().$field(), proxy);
              // }

              // ui.add_space(4.0);

              ui.vertical(|ui| {
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

            ui.add_space(2.0);
          )*

          ui.add_space(24.0);
        }
      }

      pub trait [<$name Proxy>] {
        $(
          #[allow(clippy::ptr_arg)]
          fn [<update_ $field>](&mut self, value: &$type) {}
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
        pub type [<$ty:camel Opts>] = NumericOpts<$ty>;
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
pub struct StringOpts {
  #[default = false]
  pub inline: bool,
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

  #[default = false]
  pub percentage: bool,

  #[default = false]
  pub inline: bool,

  #[default = true]
  pub slider: bool,
}
