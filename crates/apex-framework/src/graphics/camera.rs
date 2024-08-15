use glam::{vec3, Mat4, Quat, Vec3};

pub trait Transformation {
  fn apply(&self) -> Mat4;
}

// Camera
pub trait Camera: Transformation {
  fn get_position(&self) -> Vec3;
  fn get_scale(&mut self) -> Vec3;
  fn get_rotation(&self) -> Quat;

  fn set_position(&mut self, value: Vec3);
  fn set_scale(&mut self, value: Vec3);
  fn set_rotation(&mut self, value: Quat);

  fn get_x(&self) -> f32;
  fn get_y(&self) -> f32;
  fn get_z(&self) -> f32;
  fn set_x(&mut self, value: f32);
  fn set_y(&mut self, value: f32);
  fn set_z(&mut self, value: f32);
}

/* Camera 2D */
pub struct Camera2D {
  position: Vec3,
  rotation: Quat,
  scale: Vec3,
}

impl Camera2D {
  pub fn new<V, Q>(position: V, rotation: Q, scale: V) -> Self
  where
    V: Into<Vec3>,
    Q: Into<Quat>,
  {
    return Self {
      position: position.into(),
      rotation: rotation.into(),
      scale: scale.into(),
    };
  }
}

#[rustfmt::skip]
impl Camera for Camera2D {
  fn get_position(&self) -> Vec3  { return self.position; }
  fn get_scale(&mut self) -> Vec3 { return self.scale;    }
  fn get_rotation(&self) -> Quat  { return self.rotation  }

  fn set_position(&mut self, value: Vec3) { self.position = value; }
  fn set_scale(&mut self, value: Vec3)    { self.scale    = value; }
  fn set_rotation(&mut self, value: Quat) { self.rotation = value; }

  fn get_x(&self) -> f32 { return self.position.x; }
  fn get_y(&self) -> f32 { return self.position.y; }
  fn get_z(&self) -> f32 { return self.position.z; }
  fn set_x(&mut self, value: f32) { self.position.x = value; }
  fn set_y(&mut self, value: f32) { self.position.y = value; }
  fn set_z(&mut self, value: f32) { self.position.z = value; }
}

impl Transformation for Camera2D {
  fn apply(&self) -> Mat4 {
    #[rustfmt::skip] let model
      = Mat4::from_scale(vec3(self.scale.x, self.scale.y, self.scale.z))
      * Mat4::from_translation(self.position)
      * Mat4::from_rotation_translation(self.rotation, Vec3::ONE);

    return model;
  }
}

// Projection
pub trait Projection: Transformation {
  fn resize(&mut self, width: u32, height: u32);
}

/* Orthographic projection matrix */
#[rustfmt::skip]
pub struct ProjectionOrthographic {
  width  : f32,
  height : f32,
  znear  : f32,
  zfar   : f32,
}

impl ProjectionOrthographic {
  pub fn new(width: impl Into<f32>, height: impl Into<f32>, znear: f32, zfar: f32) -> Self {
    #[rustfmt::skip] return Self {
      width  : width.into(),
      height : height.into(),
      znear  : znear,
      zfar   : zfar,
    };
  }
}

impl Projection for ProjectionOrthographic {
  fn resize(&mut self, width: u32, height: u32) {
    self.width = width as f32;
    self.height = height as f32;
  }
}

impl Transformation for ProjectionOrthographic {
  fn apply(&self) -> Mat4 {
    return Mat4::orthographic_lh(0.0, self.width, self.height, 0.0, self.znear, self.zfar);
  }
}
