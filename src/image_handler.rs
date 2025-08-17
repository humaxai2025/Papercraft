use anyhow::{Context, Result};
use image::{ImageFormat, DynamicImage, GenericImageView};
use printpdf::{Image as PdfImage, ImageTransform, PdfLayerReference, Mm, Px};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use url::Url;

pub struct ImageHandler {
    client: reqwest::Client,
}

impl ImageHandler {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn add_image_to_pdf(
        &self,
        layer: &PdfLayerReference,
        image_url: &str,
        x: Mm,
        y: Mm,
        max_width: Mm,
        max_height: Mm,
    ) -> Result<Mm> {
        // Load the image
        let image_data = if self.is_url(image_url) {
            self.load_image_from_url(image_url).await?
        } else {
            self.load_image_from_file(image_url)?
        };

        // Calculate appropriate dimensions
        let (width, height) = self.calculate_dimensions(&image_data, max_width, max_height);
        
        // Convert image to PDF format and add to layer
        self.add_image_to_layer(layer, image_data, x, y, width, height)?;
        
        // Return the height used by the image
        Ok(height)
    }

    pub fn add_image_to_pdf_sync(
        &self,
        layer: &PdfLayerReference,
        image_path: &str,
        x: Mm,
        y: Mm,
        max_width: Mm,
        max_height: Mm,
    ) -> Result<Mm> {
        let image_data = if self.is_url(image_path) {
            // For URLs, use blocking download
            self.download_image_sync(image_path)?
        } else {
            self.load_image_from_file(image_path)?
        };

        let (width, height) = self.calculate_dimensions(&image_data, max_width, max_height);
        self.add_image_to_layer(layer, image_data, x, y, width, height)?;
        
        Ok(height)
    }

    fn download_image_sync(&self, url: &str) -> Result<DynamicImage> {
        // Use blocking HTTP request for image download
        let response = reqwest::blocking::get(url)
            .context("Failed to download image from URL")?;
        
        let bytes = response.bytes()
            .context("Failed to read image bytes from response")?;
        
        let image = image::load_from_memory(&bytes)
            .context("Failed to decode downloaded image")?;
        
        Ok(image)
    }

    fn add_image_to_layer(
        &self,
        layer: &PdfLayerReference,
        image: DynamicImage,
        x: Mm,
        y: Mm,
        width: Mm,
        height: Mm,
    ) -> Result<()> {
        // Convert image to RGB format
        let rgb_image = image.to_rgb8();
        let (img_width, img_height) = rgb_image.dimensions();
        
        // Create image data vector
        let mut image_data = Vec::new();
        for pixel in rgb_image.pixels() {
            image_data.push(pixel[0]); // R
            image_data.push(pixel[1]); // G  
            image_data.push(pixel[2]); // B
        }

        // Create PDF image object
        let pdf_image = PdfImage::try_from(printpdf::ImageXObject {
            width: Px(img_width as usize),
            height: Px(img_height as usize),
            color_space: printpdf::ColorSpace::Rgb,
            bits_per_component: printpdf::ColorBits::Bit8,
            interpolate: true,
            image_data,
            image_filter: None,
            clipping_bbox: None,
        })?;

        // Calculate scale factors  
        let scale_x = width.0 / (img_width as f32 / 72.0 * 25.4); // Convert pixels to mm
        let scale_y = height.0 / (img_height as f32 / 72.0 * 25.4);

        // Create transform
        let transform = ImageTransform {
            translate_x: Some(x),
            translate_y: Some(y - height), // PDF coordinates are bottom-left origin
            scale_x: Some(scale_x),
            scale_y: Some(scale_y),
            ..Default::default()
        };
        
        // Add image to layer
        pdf_image.add_to_layer(layer.clone(), transform);
        
        Ok(())
    }

    async fn load_image_from_url(&self, url: &str) -> Result<DynamicImage> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to download image")?;
        
        let bytes = response
            .bytes()
            .await
            .context("Failed to read image bytes")?;
        
        let image = image::load_from_memory(&bytes)
            .context("Failed to decode downloaded image")?;
        
        Ok(image)
    }

    fn load_image_from_file(&self, path: &str) -> Result<DynamicImage> {
        if !Path::new(path).exists() {
            anyhow::bail!("Image file not found: {}", path);
        }
        
        let file = File::open(path)
            .with_context(|| format!("Failed to open image file: {}", path))?;
        
        let reader = BufReader::new(file);
        let format = self.detect_format(path)?;
        
        let image = image::load(reader, format)
            .with_context(|| format!("Failed to load image: {}", path))?;
        
        Ok(image)
    }

    fn is_url(&self, path: &str) -> bool {
        Url::parse(path).is_ok() && (path.starts_with("http://") || path.starts_with("https://"))
    }

    fn detect_format(&self, path: &str) -> Result<ImageFormat> {
        let extension = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "png" => Ok(ImageFormat::Png),
            "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
            "gif" => Ok(ImageFormat::Gif),
            "bmp" => Ok(ImageFormat::Bmp),
            "tiff" | "tif" => Ok(ImageFormat::Tiff),
            "webp" => Ok(ImageFormat::WebP),
            _ => anyhow::bail!("Unsupported image format: {}", extension),
        }
    }

    fn calculate_dimensions(
        &self,
        image: &DynamicImage,
        max_width: Mm,
        max_height: Mm,
    ) -> (Mm, Mm) {
        let (img_width, img_height) = image.dimensions();
        let img_width = img_width as f32;
        let img_height = img_height as f32;
        let aspect_ratio = img_width / img_height;

        // Convert image size from pixels to mm (assuming 96 DPI)
        let img_width_mm = img_width * 25.4 / 96.0;
        let img_height_mm = img_height * 25.4 / 96.0;

        // Check if image fits within bounds
        if img_width_mm <= max_width.0 && img_height_mm <= max_height.0 {
            // Image fits, use original size
            (Mm(img_width_mm), Mm(img_height_mm))
        } else if img_width_mm / max_width.0 > img_height_mm / max_height.0 {
            // Width is the limiting factor
            (max_width, Mm(max_width.0 / aspect_ratio))
        } else {
            // Height is the limiting factor
            (Mm(max_height.0 * aspect_ratio), max_height)
        }
    }

    pub fn placeholder_for_image(&self, url: &str) -> String {
        format!("[Image: {}]", url)
    }
}