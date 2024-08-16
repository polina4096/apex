use apex_framework::graphics::{graphics::Graphics, origin::Origin, sprite_renderer::sprite_renderer::SpriteRenderer};
use glam::{vec2, Vec2};
use image::GenericImageView as _;
use instant::Instant;

use crate::client::gameplay::taiko_player::TaikoInput;

pub struct HitDrumDisplay {
  drum_background_sprite: usize,
  drum_inner_left_sprite: usize,
  drum_inner_right_sprite: usize,
  drum_outer_left_sprite: usize,
  drum_outer_right_sprite: usize,

  drum_background_size: Vec2,
  drum_inner_size: Vec2,
  drum_outer_size: Vec2,

  last_hit_inner_right: Instant,
  last_hit_inner_left: Instant,
  last_hit_outer_right: Instant,
  last_hit_outer_left: Instant,

  pos_x: f32,
  pos_y: f32,
  gameplay_scale: f32,
}

impl HitDrumDisplay {
  pub fn new(
    graphics: &Graphics,
    sprite_renderer: &mut SpriteRenderer,
    pos_x: f32,
    pos_y: f32,
    gameplay_scale: f32,
  ) -> Self {
    let (x, y) = (pos_x, pos_y);
    let origin = Origin::CenterCenter;

    let image = image::open("./assets/drum_background.png").unwrap();
    let drum_background_size = vec2(image.dimensions().0 as f32, image.dimensions().1 as f32);
    let texture = sprite_renderer.add_texture(&graphics.device, &graphics.queue, &image);
    let drum_background_sprite = sprite_renderer.alloc_sprite(
      &graphics.device,
      vec2(x, y),
      drum_background_size * gameplay_scale,
      origin,
      false,
      false,
      texture,
    );

    let image = image::open("./assets/drum_inner.png").unwrap();
    let drum_inner_size = vec2(image.dimensions().0 as f32, image.dimensions().1 as f32);
    let drum_inner_texture = sprite_renderer.add_texture(&graphics.device, &graphics.queue, &image);

    let image = image::open("./assets/drum_outer.png").unwrap();
    let drum_outer_size = vec2(image.dimensions().0 as f32, image.dimensions().1 as f32);
    let drum_outer_texture = sprite_renderer.add_texture(&graphics.device, &graphics.queue, &image);

    let drum_inner_left_sprite = sprite_renderer.alloc_sprite(
      &graphics.device,
      vec2(x - drum_inner_size.x * gameplay_scale / 2.0, y),
      drum_inner_size * gameplay_scale,
      origin,
      false,
      false,
      drum_inner_texture,
    );

    let drum_inner_right_sprite = sprite_renderer.alloc_sprite(
      &graphics.device,
      vec2(x + drum_inner_size.x * gameplay_scale / 2.0, y),
      drum_inner_size * gameplay_scale,
      origin,
      true,
      false,
      drum_inner_texture,
    );

    let drum_outer_left_sprite = sprite_renderer.alloc_sprite(
      &graphics.device,
      vec2(x - drum_outer_size.x * gameplay_scale / 2.0, y),
      drum_outer_size * gameplay_scale,
      origin,
      true,
      false,
      drum_outer_texture,
    );

    let drum_outer_right_sprite = sprite_renderer.alloc_sprite(
      &graphics.device,
      vec2(x + drum_outer_size.x * gameplay_scale / 2.0, y),
      drum_outer_size * gameplay_scale,
      origin,
      false,
      false,
      drum_outer_texture,
    );

    sprite_renderer.mutate_sprite(&graphics.device, drum_inner_left_sprite, |model| {
      model.color.a = 0.0;
    });

    sprite_renderer.mutate_sprite(&graphics.device, drum_inner_right_sprite, |model| {
      model.color.a = 0.0;
    });

    sprite_renderer.mutate_sprite(&graphics.device, drum_outer_left_sprite, |model| {
      model.color.a = 0.0;
    });

    sprite_renderer.mutate_sprite(&graphics.device, drum_outer_right_sprite, |model| {
      model.color.a = 0.0;
    });

    return Self {
      drum_background_sprite,
      drum_inner_left_sprite,
      drum_inner_right_sprite,
      drum_outer_left_sprite,
      drum_outer_right_sprite,

      drum_background_size,
      drum_inner_size,
      drum_outer_size,

      last_hit_inner_right: Instant::now(),
      last_hit_inner_left: Instant::now(),
      last_hit_outer_right: Instant::now(),
      last_hit_outer_left: Instant::now(),

      pos_x,
      pos_y,
      gameplay_scale,
    };
  }

  pub fn prepare(&mut self, graphics: &Graphics, sprite_renderer: &mut SpriteRenderer) {
    self.update_drum(
      graphics,
      sprite_renderer,
      self.drum_inner_left_sprite,
      self.last_hit_inner_left.elapsed().as_secs_f32(),
    );

    self.update_drum(
      graphics,
      sprite_renderer,
      self.drum_inner_right_sprite,
      self.last_hit_inner_right.elapsed().as_secs_f32(),
    );

    self.update_drum(
      graphics,
      sprite_renderer,
      self.drum_outer_left_sprite,
      self.last_hit_outer_left.elapsed().as_secs_f32(),
    );

    self.update_drum(
      graphics,
      sprite_renderer,
      self.drum_outer_right_sprite,
      self.last_hit_outer_right.elapsed().as_secs_f32(),
    );
  }

  fn update_drum(&mut self, graphics: &Graphics, sprite_renderer: &mut SpriteRenderer, sprite_id: usize, elapsed: f32) {
    let anim_fade_duration = 0.15;

    if elapsed < anim_fade_duration + 0.1 {
      sprite_renderer.mutate_sprite(&graphics.device, sprite_id, |model| {
        model.color.a = 1.0 - (elapsed / anim_fade_duration).min(1.0);
      });
    }
  }

  pub fn hit(&mut self, input: TaikoInput) {
    let now = Instant::now();

    match input {
      TaikoInput::KatLeft => {
        self.last_hit_outer_right = now;
      }

      TaikoInput::KatRight => {
        self.last_hit_outer_left = now;
      }

      TaikoInput::DonRight => {
        self.last_hit_inner_left = now;
      }

      TaikoInput::DonLeft => {
        self.last_hit_inner_right = now;
      }
    }
  }

  pub fn set_gameplay_scale(&mut self, device: &wgpu::Device, sprite_renderer: &mut SpriteRenderer, value: f32) {
    self.gameplay_scale = value;

    sprite_renderer.mutate_sprite(device, self.drum_background_sprite, |model| {
      model.scale = self.drum_background_size * self.gameplay_scale;
    });

    sprite_renderer.mutate_sprite(device, self.drum_inner_left_sprite, |model| {
      model.position.x = self.pos_x - self.drum_inner_size.x * self.gameplay_scale / 2.0;
      model.scale = self.drum_inner_size * self.gameplay_scale;
    });

    sprite_renderer.mutate_sprite(device, self.drum_inner_right_sprite, |model| {
      model.position.x = self.pos_x + self.drum_inner_size.x * self.gameplay_scale / 2.0;
      model.scale = self.drum_inner_size * self.gameplay_scale;
    });

    sprite_renderer.mutate_sprite(device, self.drum_outer_left_sprite, |model| {
      model.position.x = self.pos_x - self.drum_outer_size.x * self.gameplay_scale / 2.0;
      model.scale = self.drum_outer_size * self.gameplay_scale;
    });

    sprite_renderer.mutate_sprite(device, self.drum_outer_right_sprite, |model| {
      model.position.x = self.pos_x + self.drum_outer_size.x * self.gameplay_scale / 2.0;
      model.scale = self.drum_outer_size * self.gameplay_scale;
    });
  }

  pub fn set_hit_position_x(&mut self, device: &wgpu::Device, sprite_renderer: &mut SpriteRenderer, value: f32) {
    self.pos_x = value;

    sprite_renderer.mutate_sprite(device, self.drum_background_sprite, |model| {
      model.position.x = value;
    });

    sprite_renderer.mutate_sprite(device, self.drum_inner_left_sprite, |model| {
      model.position.x = value - self.drum_inner_size.x * self.gameplay_scale / 2.0;
    });

    sprite_renderer.mutate_sprite(device, self.drum_inner_right_sprite, |model| {
      model.position.x = value + self.drum_inner_size.x * self.gameplay_scale / 2.0;
    });

    sprite_renderer.mutate_sprite(device, self.drum_outer_left_sprite, |model| {
      model.position.x = value - self.drum_outer_size.x * self.gameplay_scale / 2.0;
    });

    sprite_renderer.mutate_sprite(device, self.drum_outer_right_sprite, |model| {
      model.position.x = value + self.drum_outer_size.x * self.gameplay_scale / 2.0;
    });
  }

  pub fn set_hit_position_y(&mut self, device: &wgpu::Device, sprite_renderer: &mut SpriteRenderer, value: f32) {
    self.pos_y = value;

    sprite_renderer.mutate_sprite(device, self.drum_background_sprite, |model| {
      model.position.y = value;
    });

    sprite_renderer.mutate_sprite(device, self.drum_inner_left_sprite, |model| {
      model.position.y = value;
    });

    sprite_renderer.mutate_sprite(device, self.drum_inner_right_sprite, |model| {
      model.position.y = value;
    });

    sprite_renderer.mutate_sprite(device, self.drum_outer_left_sprite, |model| {
      model.position.y = value;
    });

    sprite_renderer.mutate_sprite(device, self.drum_outer_right_sprite, |model| {
      model.position.y = value;
    });
  }
}
