use crate::core::{app::App, core::Core};

pub trait Action<A: App> {
  fn execute(app: &mut A, core: &mut Core<A>, repeat: bool) -> bool;
}

#[macro_export]
macro_rules! actions {
  ($name:ident<$app:ty> {
    $($action:ident),+ $(,)?
  }) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum $name {
      $($action),+
    }

    impl $name {
      /// Returns true if the execution was successful and the input should be consumed.
      pub fn execute(self, client: &mut $app, core: &mut $crate::core::core::Core<$app>, repeat: bool) -> bool {
        use $crate::client::action::action::Action;

        match self {
          $(
            $name::$action => {
              $action::execute(client, core, repeat)
            }
          )+
        }
      }
    }
  };
}
