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
        path: std::path::PathBuf,

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
            path: std::path::PathBuf::from("./config.toml"),
            $($category: [<$category:camel Settings>]::default(),)+
          };
        }
      }

      impl Settings {
        pub fn default_with_path(path: impl AsRef<std::path::Path>) -> Self {
          return Self {
            path: path.as_ref().to_owned(),
            $($category: [<$category:camel Settings>]::default(),)+
          };
        }

        pub fn from_file(path: impl AsRef<std::path::Path>) -> Self {
          return std::fs::read_to_string(&path)
            .map(|data| {
              return toml::from_str(&data).unwrap_or_else(|e| {
                log::error!("Failed to parse config file, falling back to default config: {}", e);

                return Settings::default_with_path(&path);
              });
            })
            .unwrap_or_else(|e| {
              let default = Settings::default_with_path(&path);

              match e.kind() {
                std::io::ErrorKind::NotFound => {
                  log::info!("Failed to open config file, file not found. Creating a default config file...");
                  let default_data = toml::to_string_pretty(&default).expect("Failed to serialize default config");
                  if let Err(e) = std::fs::write(&path, default_data) {
                    log::error!("Failed to write default config file: {}", e);
                  }
                }

                std::io::ErrorKind::PermissionDenied => {
                  log::info!("Failed to open config file, insufficient permissions. Falling back to default configuration.");
                }

                _ => {
                  log::error!("Failed to access config file: {}. Falling back to default configuration.", e);
                }
              }

              return default;
            });
        }
      }

      impl Drop for Settings {
        fn drop(&mut self) {
          let data = match toml::to_string_pretty(&self) {
            Ok(data) => data,
            Err(e) => {
              log::error!("Failed to serialize settings: {}", e);
              return
            }
          };

          if let Err(e) = std::fs::write(&self.path, data) {
            log::error!("Failed to write settings to file: {}", e);
            return;
          }

          let path = self.path.canonicalize().unwrap_or(self.path.clone());
          log::info!("Settings successfully written to `{}`", path.display());
        }
      }

      pub trait SettingsProxy {
        $($(fn [<update_ $category _ $setting>](&mut self, _value: $type) {})+)+
      }
    }
  };
}
