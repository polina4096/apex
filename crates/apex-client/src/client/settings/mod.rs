pub mod graphics;
pub mod proxy;
pub mod settings;

#[macro_export]
macro_rules! settings {
  (
    $(
      $(#[$($attrs_category:tt)*])*
      $category:ident {
        $(
          $(#[$($attrs_setting:tt)*])*
          $setting:ident: $type:ty = $default_value:expr
        )+
      }
    )+
  ) => {
    paste::paste! {
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct Settings {
        $(pub $category: [<$category:camel Settings>],)+
      }

      $(
        $(#[$($attrs_category)*])*
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct [<$category:camel Settings>] {
          $(
            $(#[$($attrs_setting)*])*
            $setting: $type,
          )+
        }

        impl Default for [<$category:camel Settings>] {
          fn default() -> Self {
            return Self {
              $($setting: $default_value,)+
            };
          }
        }

        impl [<$category:camel Settings>] {
          $(
            pub fn $setting(&self) -> $type {
              return self.$setting;
            }

            pub fn [<set_ $setting>](&mut self, value: $type, proxy: &mut impl SettingsProxy) {
              self.$setting = value;
              proxy.[<update_ $category _ $setting>](value);
            }
          )+
        }
      )+

      impl Default for Settings {
        fn default() -> Self {
          return Self {
            $($category: [<$category:camel Settings>]::default(),)+
          };
        }
      }

      pub trait SettingsProxy {
        $($(fn [<update_ $category _ $setting>](&mut self, _value: $type) {})+)+
      }
    }
  };
}
