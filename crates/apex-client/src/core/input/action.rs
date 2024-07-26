use crate::core::{app::App, core::Core};

pub trait Action<A: App> {
  fn execute(app: &mut A, core: &mut Core<A>, repeat: bool) -> bool;
}

#[macro_export]
macro_rules! actions {
  ($name:ident<$app:ty> {
    $(
      #[doc = $action_desc:expr]
      $action:ident $(as $action_name:literal)? = $action_comb:expr
    ),+ $(,)?
  }) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum $name {
      $($action),+
    }

    impl $name {
      /// Returns true if the execution was successful and the input should be consumed.
      pub fn execute(self, client: &mut $app, core: &mut $crate::core::core::Core<$app>, repeat: bool) -> bool {
        use $crate::core::input::action::Action;

        match self {
          $(
            $name::$action => {
              $action::execute(client, core, repeat)
            }
          )+
        }
      }

      pub fn insert_keybinds(keybinds: &mut $crate::core::input::keybinds::Keybinds<$name>) {
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
    }
  };
}
