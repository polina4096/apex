use apex_framework::graphics::{
  graphics::Graphics,
  origin::Origin,
  sprite_renderer::sprite_renderer::{AllocId, SpriteRenderer},
};
use glam::{vec2, Vec2};
use image::GenericImageView as _;
use instant::Instant;

use crate::client::score::judgement_processor::Judgement;

pub struct HitResultDisplay {
  hit_result_sprite: usize,

  gameplay_scale: f32,

  judgement_150_atlas_texture: AllocId,
  judgement_miss_atlas_texture: AllocId,
  last_hit_judgement: Judgement,
  last_hit_judgement_time: Instant,
  judgement_150_size: Vec2,
  judgement_miss_size: Vec2,
}

impl HitResultDisplay {
  pub fn new(
    graphics: &Graphics,
    sprite_renderer: &mut SpriteRenderer,
    hit_pos_x: f32,
    hit_pos_y: f32,
    gameplay_scale: f32,
  ) -> Self {
    let (x, y) = (hit_pos_x, hit_pos_y);
    let origin = Origin::CenterCenter;

    let judgement_150_image = image::open("./assets/judgement_150.png").unwrap();
    let (width, height) = (judgement_150_image.dimensions().0 as f32, judgement_150_image.dimensions().1 as f32);
    let judgement_150_size = vec2(width, height);

    let judgement_miss_image = image::open("./assets/judgement_miss.png").unwrap();
    let (width, height) = (judgement_miss_image.dimensions().0 as f32, judgement_miss_image.dimensions().1 as f32);
    let judgement_miss_size = vec2(width, height);

    let [judgement_150_atlas_texture, judgement_miss_atlas_texture] = sprite_renderer.add_textures([
      //
      &judgement_150_image,
      &judgement_miss_image,
    ]);

    let size = judgement_miss_size * gameplay_scale;
    let texture = judgement_miss_atlas_texture;
    let hit_result_sprite = sprite_renderer.alloc_sprite(
      //
      &graphics.device,
      vec2(x, y),
      size,
      origin,
      false,
      false,
      texture,
    );

    return Self {
      hit_result_sprite,
      gameplay_scale,
      judgement_150_atlas_texture,
      judgement_miss_atlas_texture,
      last_hit_judgement: Judgement::Miss,
      last_hit_judgement_time: Instant::now(),
      judgement_150_size,
      judgement_miss_size,
    };
  }

  pub fn prepare(&mut self, graphics: &Graphics, sprite_renderer: &mut SpriteRenderer) {
    let elapsed = self.last_hit_judgement_time.elapsed().as_secs_f32();
    let anim_fade_duration = 0.35;

    if elapsed < anim_fade_duration + 0.1 {
      sprite_renderer.mutate_sprite(&graphics.device, self.hit_result_sprite, |model| {
        model.color.a = 1.0 - (elapsed / anim_fade_duration).min(1.0);

        let size = match self.last_hit_judgement {
          Judgement::Hit150 => self.judgement_150_size,
          Judgement::Miss => self.judgement_miss_size,
          _ => Vec2::ZERO,
        };

        let anim_scale_multiplier = 0.4;
        let anim_value_multiplier = 1.0 + (elapsed * anim_scale_multiplier * 2.0).min(anim_scale_multiplier);
        model.scale = size * anim_value_multiplier * self.gameplay_scale;
      });
    }
  }

  pub fn reset(&mut self, graphics: &Graphics, sprite_renderer: &mut SpriteRenderer) {
    sprite_renderer.mutate_sprite(&graphics.device, self.hit_result_sprite, |model| {
      model.scale = Vec2::ZERO;
    });
  }

  pub fn update_hit_result(&mut self, graphics: &Graphics, sprite_renderer: &mut SpriteRenderer, judgement: Judgement) {
    self.last_hit_judgement_time = Instant::now();
    self.last_hit_judgement = judgement;

    match judgement {
      Judgement::Hit300 => {
        sprite_renderer.mutate_sprite(&graphics.device, self.hit_result_sprite, |model| {
          model.scale = Vec2::ZERO;
        });
      }

      Judgement::Hit150 => {
        let (offset, size) = sprite_renderer.uv_pairs(self.judgement_150_atlas_texture);
        sprite_renderer.mutate_sprite(&graphics.device, self.hit_result_sprite, |model| {
          model.scale = vec2(self.judgement_150_size.x, self.judgement_150_size.y) * self.gameplay_scale;
          model.uv_offset = offset;
          model.uv_scale = size;
        });
      }

      Judgement::Miss => {
        let (offset, size) = sprite_renderer.uv_pairs(self.judgement_miss_atlas_texture);
        sprite_renderer.mutate_sprite(&graphics.device, self.hit_result_sprite, |model| {
          model.scale = vec2(self.judgement_miss_size.x, self.judgement_miss_size.y) * self.gameplay_scale;
          model.uv_offset = offset;
          model.uv_scale = size;
        });
      }
    }
  }

  pub fn set_gameplay_scale(&mut self, value: f32) {
    self.gameplay_scale = value;
  }

  pub fn set_hit_position_x(&mut self, device: &wgpu::Device, sprite_renderer: &mut SpriteRenderer, value: f32) {
    sprite_renderer.mutate_sprite(device, self.hit_result_sprite, |model| {
      model.position.x = value;
    });
  }

  pub fn set_hit_position_y(&mut self, device: &wgpu::Device, sprite_renderer: &mut SpriteRenderer, value: f32) {
    sprite_renderer.mutate_sprite(device, self.hit_result_sprite, |model| {
      model.position.y = value;
    });
  }
}
