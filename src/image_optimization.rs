use anyhow::{Context, Result};
use image::{ImageFormat, DynamicImage, imageops::FilterType, GenericImageView};
use regex::Regex;
use std::path::{Path, PathBuf};
use base64::{engine::general_purpose::STANDARD, Engine};
use crate::config::ImageConfig;

pub struct ImageOptimizer {
    config: ImageConfig,
    processed_images: std::collections::HashMap<String, String>,
}

impl ImageOptimizer {
    pub fn new(config: ImageConfig) -> Self {
        Self {
            config,
            processed_images: std::collections::HashMap::new(),
        }
    }

    pub fn process_images_in_html(&mut self, html: &str, base_path: &Path) -> Result<String> {
        if !self.config.optimization {
            return Ok(html.to_string());
        }

        // Process both img tags and markdown-style images
        let img_regex = Regex::new(r#"<img([^>]*?)src="([^"]+)"([^>]*?)>"#).unwrap();
        let markdown_img_regex = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();

        let mut processed_html = html.to_string();

        // Process HTML img tags
        processed_html = img_regex.replace_all(&processed_html, |caps: &regex::Captures| {
            let before_src = &caps[1];
            let src = &caps[2];
            let after_src = &caps[3];

            match self.process_image_src(src, base_path) {
                Ok(new_src) => {
                    format!(r#"<img{}src="{}"{}>"#, before_src, new_src, after_src)
                }
                Err(_) => caps[0].to_string() // Keep original on error
            }
        }).to_string();

        // Process Markdown images (convert to HTML with optimization)
        processed_html = markdown_img_regex.replace_all(&processed_html, |caps: &regex::Captures| {
            let alt = &caps[1];
            let src = &caps[2];

            match self.process_image_src(src, base_path) {
                Ok(new_src) => {
                    format!(r#"<img src="{}" alt="{}" class="markdown-image">"#, new_src, alt)
                }
                Err(_) => caps[0].to_string() // Keep original on error
            }
        }).to_string();

        Ok(processed_html)
    }

    fn process_image_src(&mut self, src: &str, base_path: &Path) -> Result<String> {
        // Skip if already processed
        if let Some(cached) = self.processed_images.get(src) {
            return Ok(cached.clone());
        }

        // Skip URLs (web images)
        if src.starts_with("http://") || src.starts_with("https://") {
            return Ok(src.to_string());
        }

        // Skip data URLs
        if src.starts_with("data:") {
            return Ok(src.to_string());
        }

        // Resolve relative path
        let image_path = if Path::new(src).is_absolute() {
            PathBuf::from(src)
        } else {
            base_path.join(src)
        };

        if !image_path.exists() {
            return Ok(src.to_string()); // Keep original if file doesn't exist
        }

        // Load and process image
        let img = image::open(&image_path)
            .with_context(|| format!("Failed to load image: {}", image_path.display()))?;

        let processed_img = self.optimize_image(img)?;
        let data_url = self.convert_to_data_url(processed_img, &image_path)?;

        // Cache the result
        self.processed_images.insert(src.to_string(), data_url.clone());

        Ok(data_url)
    }

    fn optimize_image(&self, mut img: DynamicImage) -> Result<DynamicImage> {
        let (width, height) = img.dimensions();

        // Resize if needed
        let needs_resize = if let Some(max_width) = self.config.max_width {
            width > max_width
        } else {
            false
        } || if let Some(max_height) = self.config.max_height {
            height > max_height
        } else {
            false
        };

        if needs_resize {
            let max_width = self.config.max_width.unwrap_or(u32::MAX);
            let max_height = self.config.max_height.unwrap_or(u32::MAX);

            // Calculate new dimensions while maintaining aspect ratio
            let aspect_ratio = width as f32 / height as f32;
            
            let (new_width, new_height) = if width > max_width || height > max_height {
                if width as f32 / max_width as f32 > height as f32 / max_height as f32 {
                    // Width is the limiting factor
                    (max_width, (max_width as f32 / aspect_ratio) as u32)
                } else {
                    // Height is the limiting factor
                    ((max_height as f32 * aspect_ratio) as u32, max_height)
                }
            } else {
                (width, height)
            };

            img = img.resize(new_width, new_height, FilterType::Lanczos3);
        }

        Ok(img)
    }

    fn convert_to_data_url(&self, img: DynamicImage, original_path: &Path) -> Result<String> {
        let format = if let Some(ref target_format) = self.config.format {
            match target_format.to_lowercase().as_str() {
                "webp" => ImageFormat::WebP,
                "png" => ImageFormat::Png,
                "jpeg" | "jpg" => ImageFormat::Jpeg,
                _ => self.detect_format_from_path(original_path)?,
            }
        } else {
            self.detect_format_from_path(original_path)?
        };

        let mut buffer = Vec::new();
        
        match format {
            ImageFormat::Jpeg => {
                let quality = self.config.quality.unwrap_or(85);
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality);
                img.write_with_encoder(encoder)
                    .context("Failed to encode JPEG")?;
            }
            ImageFormat::Png => {
                img.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Png)
                    .context("Failed to encode PNG")?;
            }
            ImageFormat::WebP => {
                // For WebP, we'll fall back to PNG since WebP encoding is complex
                img.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Png)
                    .context("Failed to encode image as PNG (WebP fallback)")?;
            }
            _ => {
                img.write_to(&mut std::io::Cursor::new(&mut buffer), format)
                    .context("Failed to encode image")?;
            }
        }

        let mime_type = match format {
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Png => "image/png",
            ImageFormat::WebP => "image/webp",
            ImageFormat::Gif => "image/gif",
            _ => "image/png", // Default fallback
        };

        let encoded = STANDARD.encode(&buffer);
        Ok(format!("data:{};base64,{}", mime_type, encoded))
    }

    fn detect_format_from_path(&self, path: &Path) -> Result<ImageFormat> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("jpg") | Some("jpeg") => Ok(ImageFormat::Jpeg),
            Some("png") => Ok(ImageFormat::Png),
            Some("gif") => Ok(ImageFormat::Gif),
            Some("webp") => Ok(ImageFormat::WebP),
            Some("bmp") => Ok(ImageFormat::Bmp),
            Some("tiff") | Some("tif") => Ok(ImageFormat::Tiff),
            _ => Ok(ImageFormat::Png), // Default fallback
        }
    }

    #[allow(dead_code)]
    pub fn add_image_styles(&self) -> String {
        r#"
/* Image optimization styles */
.markdown-image {
    max-width: 100%;
    height: auto;
    display: block;
    margin: 1rem auto;
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.image-caption {
    text-align: center;
    font-style: italic;
    font-size: 0.9em;
    color: #666;
    margin-top: 0.5rem;
    margin-bottom: 1rem;
}

.figure {
    margin: 2rem 0;
    text-align: center;
}

.figure img {
    max-width: 100%;
    height: auto;
}

.figure-caption {
    margin-top: 0.5rem;
    font-size: 0.9em;
    color: #666;
    font-style: italic;
}
"#.to_string()
    }
}