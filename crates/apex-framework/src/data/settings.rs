#[macro_export]
macro_rules! settings_ref_ty_helper {
  (, $type:ty) => {
    $type
  };

  ($borrowed_ty:ty, $type:ty) => {
    $borrowed_ty
  };
}

#[macro_export]
macro_rules! settings_ref_borrowed_ty_helper {
  (, $type:ty) => {
    $type
  };

  ($borrowed_ty:ty, $type:ty) => {
    &$type
  };
}

#[macro_export]
macro_rules! settings_ref_borrow_helper {
  (, $e:expr) => {
    $e
  };

  ($borrowed_ty:ty, $e:expr) => {
    &$e
  };
}

#[macro_export]
macro_rules! settings_ref_borrowed_setter_helper {
  (, $place:expr, $value:expr) => {
    $place = $value;
  };

  ($borrowed_ty:ty, $place:expr, $value:expr) => {
    $place.clone_from($value);
  };
}

#[macro_export]
macro_rules! settings_ref_setter_helper {
  (, $place:expr, $value:expr) => {
    $place = $value;
  };

  ($borrowed_ty:ty, $place:expr, $value:expr) => {
    $place = $value.to_owned();
  };
}

#[macro_export]
macro_rules! settings {
  (
    $(
      $(#[$($attrs_category:tt)*])*
      $category:ident {
        $(
          $(#[$($attrs_setting:tt)*])*
          $setting:ident: $type:ty $(as $borrowed_ty:ty)? = $default_value:expr,
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
            pub fn $setting(&self) -> $crate::settings_ref_ty_helper!($($borrowed_ty)?, $type) {
              return $crate::settings_ref_borrow_helper!($($borrowed_ty)?, self.$setting);
            }

            pub fn [<set_ $setting>](&mut self, value: $crate::settings_ref_ty_helper!($($borrowed_ty)?, $type), proxy: &mut impl SettingsProxy) {
              $crate::settings_ref_setter_helper!($($borrowed_ty)?, self.$setting, value);
              proxy.[<update_ $category _ $setting>]($crate::settings_ref_borrow_helper!($($borrowed_ty)?, self.$setting));
            }

            pub fn [<borrowed_ $setting>](&self) -> $crate::settings_ref_borrowed_ty_helper!($($borrowed_ty)?, $type) {
              return $crate::settings_ref_borrow_helper!($($borrowed_ty)?, self.$setting);
            }

            pub fn [<set_borrowed_ $setting>](&mut self, value: $crate::settings_ref_borrowed_ty_helper!($($borrowed_ty)?, $type), proxy: &mut impl SettingsProxy) {
              $crate::settings_ref_borrowed_setter_helper!($($borrowed_ty)?, self.$setting, value);
              proxy.[<update_ $category _ $setting>]($crate::settings_ref_borrow_helper!($($borrowed_ty)?, self.$setting));
            }
          )+
        }
      )+

      pub trait SettingsProxy {
        $($(fn [<update_ $category _ $setting>](&mut self, _value: $crate::settings_ref_ty_helper!($($borrowed_ty)?, $type)) {})+)+
      }

      impl Default for Settings {
        fn default() -> Self {
          return Self {
            $($category: [<$category:camel Settings>]::default(),)+
          };
        }
      }

      impl $crate::data::persistent::Persistent for Settings {
        fn load(path: impl AsRef<std::path::Path>) -> Self {
          {
            let path = path.as_ref().canonicalize().unwrap_or(path.as_ref().to_owned());
            log::info!("Loading settings from `{}`", path.display());
          }

          return std::fs::read_to_string(&path)
            .map(|data| {
              return toml::from_str(&data).unwrap_or_else(|e| {
                log::error!("Failed to parse config file, falling back to default config: {}", e);

                return Settings::default();
              });
            })
            .unwrap_or_else(|e| {
              let default = Settings::default();

              match e.kind() {
                std::io::ErrorKind::NotFound => {
                  log::warn!("Failed to open config file, file not found. Creating a default config file...");
                  let default_data = toml::to_string_pretty(&default).expect("Failed to serialize default config");
                  if let Err(e) = std::fs::write(&path, default_data) {
                    log::error!("Failed to write default config file: {}", e);
                  }
                }

                std::io::ErrorKind::PermissionDenied => {
                  log::warn!("Failed to open config file, insufficient permissions. Falling back to default configuration.");
                }

                _ => {
                  log::error!("Failed to access config file: {}. Falling back to default configuration.", e);
                }
              }

              return default;
            });
        }

        fn save(&self, path: impl AsRef<std::path::Path>) {
          let data = match toml::to_string_pretty(&self) {
            Ok(data) => data,
            Err(e) => {
              log::error!("Failed to serialize settings: {}", e);
              return
            }
          };

          if let Err(e) = std::fs::write(&path, data) {
            log::error!("Failed to write settings to file: {}", e);
            return;
          }

          let path = path.as_ref().canonicalize().unwrap_or(path.as_ref().to_owned());
          log::info!("Settings successfully written to `{}`", path.display());
        }
      }
    }
  };
}
