use std::path::Path;

use wcore::graphics::{texture::Texture, context::Graphics};

pub struct Skin {
    pub circle       : Texture,
    pub overlay      : Texture,
    pub big_circle   : Texture,
    pub big_overlay  : Texture,
    pub hit_position : Texture,
}

impl Skin {
    pub fn from_path(path: impl AsRef<Path>, graphics: &Graphics) -> Self {
        let path = path.as_ref();
        return Skin {
            circle       : Texture::from_path(path.to_owned().join("taikohitcircle.png"       ), graphics).unwrap_or(Texture::default(graphics)),
            overlay      : Texture::from_path(path.to_owned().join("taikohitcircleoverlay.png"), graphics).unwrap_or(Texture::default(graphics)),
            big_circle   : Texture::from_path(path.to_owned().join("taikobigcircle.png"       ), graphics).unwrap_or(Texture::default(graphics)),
            big_overlay  : Texture::from_path(path.to_owned().join("taikobigcircleoverlay.png"), graphics).unwrap_or(Texture::default(graphics)),
            hit_position : Texture::from_path(path.to_owned().join("approachcircle.png"       ), graphics).unwrap_or(Texture::default(graphics)),
        };
    }

    pub fn default(graphics: &Graphics) -> Self {
        return Skin {
            circle       : Texture::from_memory(include_bytes!("../../../res/taikohitcircle.png"       ), graphics).unwrap(),
            overlay      : Texture::from_memory(include_bytes!("../../../res/taikohitcircleoverlay.png"), graphics).unwrap(),
            big_circle   : Texture::from_memory(include_bytes!("../../../res/taikobigcircle.png"       ), graphics).unwrap(),
            big_overlay  : Texture::from_memory(include_bytes!("../../../res/taikobigcircleoverlay.png"), graphics).unwrap(),
            hit_position : Texture::from_memory(include_bytes!("../../../res/approachcircle.png"       ), graphics).unwrap(),
        };
    }
}