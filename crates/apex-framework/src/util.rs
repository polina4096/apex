use std::fmt::Debug;

/// Extension trait that provides a `catch` method similar to `unwrap`.
///
/// `unwrap` is usually used quickly prototype code, leaving in the codebase
/// makes it harder to actually find (grep) sections of code where the error
/// handling should be improved.
///
/// This trait provides a `catch` method that is similar to `unwrap`, which
/// panics if the result is an `Err` value with a standardized error message.
pub trait ResultExt<T, E> {
  fn catch(self) -> T;
}

impl<T, E: Debug> ResultExt<T, E> for Result<T, E> {
  fn catch(self) -> T {
    return self.expect("should never fail");
  }
}
