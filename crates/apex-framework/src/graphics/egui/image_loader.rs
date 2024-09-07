use std::{
  mem::size_of,
  path::Path,
  sync::{mpsc::Sender, Arc},
};

use dashmap::DashMap;
use egui::{
  load::{BytesPoll, ImageLoadResult, ImageLoader, ImagePoll, LoadError, SizeHint},
  ColorImage, Vec2,
};
use image::ImageFormat;

#[derive(Clone)]
enum ImageState {
  Pending { size: Option<Vec2> },
  Ready(Arc<ColorImage>),
  Error(LoadError),
}

pub struct BackgroundImageLoader {
  cache: Arc<DashMap<String, ImageState, ahash::RandomState>>,
  sender: Sender<String>,
}

impl BackgroundImageLoader {
  pub const ID: &'static str = egui::generate_loader_id!(BackgroundImageLoader);

  pub fn new(ctx: egui::Context) -> Self {
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let cache = Arc::new(DashMap::default());

    std::thread::spawn({
      let cache = Arc::downgrade(&cache);

      move || {
        while let Ok(uri) = rx.recv() {
          loop {
            match ctx.try_load_bytes(&uri) {
              #[expect(unused)]
              Ok(BytesPoll::Ready { bytes, mime, .. }) => {
                // (2 and 3)
                // TODO: fix this, stopped working with commit ba80038 of egui for some reason
                // if mime.as_deref().is_some_and(is_unsupported_mime) || image::guess_format(&bytes).is_err() {
                //   // Err(LoadError::NotSupported);
                //   continue;
                // }

                log::trace!("Started loading {uri:?}");
                let result = match image::load_from_memory(&bytes) {
                  Ok(image) => {
                    ImageState::Ready(Arc::new(ColorImage::from_rgba_unmultiplied(
                      [image.width() as usize, image.height() as usize],
                      &image.to_rgba8(),
                    )))
                  }

                  Err(e) => ImageState::Error(LoadError::Loading(e.to_string())),
                };

                log::trace!("finished loading {uri:?}");
                let Some(cache) = cache.upgrade() else { return };
                cache.insert(uri, result);

                break;
              }

              Err(err) => {
                let Some(cache) = cache.upgrade() else { return };
                cache.insert(uri, ImageState::Error(LoadError::Loading(err.to_string())));
                break;
              }

              _ => {}
            };
          }
        }
      }
    });

    return Self { cache, sender: tx };
  }
}

fn is_supported_uri(uri: &str) -> bool {
  let Some(ext) = Path::new(uri).extension().and_then(|ext| ext.to_str()) else {
    // `true` because if there's no extension, assume that we support it
    return true;
  };

  return ImageFormat::all()
    .filter(ImageFormat::reading_enabled)
    .flat_map(ImageFormat::extensions_str)
    .any(|x| *x == ext);
}

#[expect(unused)]
fn is_unsupported_mime(mime: &str) -> bool {
  return !ImageFormat::all()
    .filter(ImageFormat::reading_enabled)
    .flat_map(ImageFormat::extensions_str)
    .any(|x| *x == mime);
}

impl ImageLoader for BackgroundImageLoader {
  fn id(&self) -> &str {
    return Self::ID;
  }

  fn load(&self, _: &egui::Context, uri: &str, _: SizeHint) -> ImageLoadResult {
    // three stages of guessing if we support loading the image:
    // 1. URI extension
    // 2. Mime from `BytesPoll::Ready`
    // 3. image::guess_format

    // (1)
    if !is_supported_uri(uri) {
      return Err(LoadError::NotSupported);
    }

    if let Some(entry) = self.cache.get(uri) {
      return match entry.value().clone() {
        ImageState::Pending { size } => Ok(ImagePoll::Pending { size }),
        ImageState::Ready(image) => Ok(ImagePoll::Ready { image }),
        ImageState::Error(e) => Err(e),
      };
    }

    self.cache.insert(uri.to_owned(), ImageState::Pending { size: None });
    self.sender.send(uri.to_owned()).expect("Failed to send uri to background image loader");
    return Ok(ImagePoll::Pending { size: None });
  }

  fn forget(&self, uri: &str) {
    let _ = self.cache.remove(uri);
  }

  fn forget_all(&self) {
    self.cache.clear();
  }

  fn byte_size(&self) -> usize {
    return self
      .cache
      .iter()
      .map(|result| {
        match result.value() {
          ImageState::Pending { size } => {
            size
              .map(|p| {
                let len = p.x.ceil() as usize * p.y.ceil() as usize;
                return len * size_of::<egui::Color32>();
              })
              .unwrap_or(0)
          }

          ImageState::Ready(image) => image.pixels.len() * size_of::<egui::Color32>(),
          ImageState::Error(_) => 0,
        }
      })
      .sum();
  }
}
