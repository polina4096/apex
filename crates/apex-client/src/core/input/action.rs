use crate::core::{app::App, core::Core};

use super::keybinds::Keybinds;

pub trait Action<A: App> {
  fn execute(app: &mut A, core: &mut Core<A>, repeat: bool) -> bool;
}

pub trait AppActions: Sized {
  type App: App;

  fn execute(self, client: &mut Self::App, core: &mut Core<Self::App>, repeat: bool) -> bool;
  fn insert_keybinds(keybinds: &mut Keybinds<Self>);
  fn action_info(&self) -> (String, String);
}

#[macro_export]
macro_rules! actions {
  ($name:ident<$app:ty> {
    $(
      #[doc = $action_desc:expr]
      $action:ident $(as $action_name:literal)? = $action_comb:expr
    ),+ $(,)?
  }) => {
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum $name {
      $($action),+
    }

    impl $crate::core::input::action::AppActions for $name {
      type App = $app;

      /// Returns true if the execution was successful and the input should be consumed.
      fn execute(self, client: &mut $app, core: &mut $crate::core::core::Core<$app>, repeat: bool) -> bool {
        use $crate::core::input::action::Action;

        match self {
          $(
            $name::$action => {
              $action::execute(client, core, repeat)
            }
          )+
        }
      }

      fn insert_keybinds(keybinds: &mut $crate::core::input::keybinds::Keybinds<Self>) {
        $(
          keybinds.add(
            $action_comb,
            $crate::core::input::keybinds::Bind {
              id: $name::$action,
              name: String::from({
                #[allow(unused)]
                let name = stringify!($action);
                $(let name = $action_name;)?
                name
              }),
              description: String::from({
                #[allow(unused)]
                let desc = "";
                let desc = $action_desc;
                desc
              }),
            },
          );
        )+
      }

      fn action_info(&self) -> (String, String) {
        match self {
          $(
            $name::$action => {
              (
                String::from({
                  #[allow(unused)]
                  let name = stringify!($action);
                  $(let name = $action_name;)?
                  name
                }),
                String::from({
                  #[allow(unused)]
                  let desc = "";
                  let desc = $action_desc;
                  desc
                }),
              )
            }
          )+
        }
      }
    }
  };
}
