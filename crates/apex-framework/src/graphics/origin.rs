#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
  TopLeft,
  TopTop,
  TopRight,

  CenterLeft,

  #[default]
  CenterCenter,

  CenterRight,

  BottomLeft,
  BottomBottom,
  BottomRight,
}
