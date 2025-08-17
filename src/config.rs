use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub output: OutputConfig,
    pub page: PageConfig,
    pub theme: ThemeConfig,
    pub fonts: FontConfig,
    pub toc: TocConfig,
    pub images: ImageConfig,
    pub code: CodeConfig,
    pub references: ReferenceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub format: String,
    pub quality: Option<f64>,
    pub compression: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageConfig {
    pub size: PageSize,
    pub margins: Margins,
    pub orientation: Orientation,
    pub header: Option<HeaderFooterConfig>,
    pub footer: Option<HeaderFooterConfig>,
    pub page_numbers: Option<PageNumberConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSize {
    pub preset: Option<String>, // "A4", "Letter", "Legal", etc.
    pub width: Option<String>,  // Custom width in inches, mm, cm
    pub height: Option<String>, // Custom height in inches, mm, cm
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margins {
    pub top: String,
    pub right: String,
    pub bottom: String,
    pub left: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderFooterConfig {
    pub enabled: bool,
    pub template: String,
    pub height: Option<String>,
    pub font_size: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageNumberConfig {
    pub enabled: bool,
    pub format: String, // "Page {page} of {total}", "{page}/{total}", etc.
    pub position: PageNumberPosition,
    pub start_from: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageNumberPosition {
    Header,
    Footer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub css_file: Option<PathBuf>,
    pub built_in: Option<String>, // "default", "dark", "minimal", etc.
    pub custom_styles: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: Option<String>,
    pub size: Option<String>,
    pub line_height: Option<f64>,
    pub custom_fonts: Option<Vec<CustomFont>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFont {
    pub name: String,
    pub path: PathBuf,
    pub weight: Option<String>, // "normal", "bold", "100-900"
    pub style: Option<String>,  // "normal", "italic"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocConfig {
    pub enabled: bool,
    pub title: String,
    pub max_depth: u8,
    pub page_numbers: bool,
    pub links: bool,
    pub style: TocStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TocStyle {
    Simple,
    Numbered,
    Indented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub optimization: bool,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub quality: Option<u8>, // 1-100
    pub format: Option<String>, // "webp", "png", "jpeg"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeConfig {
    pub line_numbers: bool,
    pub highlight_theme: String,
    pub word_wrap: bool,
    pub show_language: bool,
    pub copy_button: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceConfig {
    pub footnotes: FootnoteConfig,
    pub bibliography: BibliographyConfig,
    pub cross_references: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteConfig {
    pub enabled: bool,
    pub style: FootnoteStyle,
    pub numbering: FootnoteNumbering,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FootnoteStyle {
    Bottom,
    End,
    Margin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FootnoteNumbering {
    Numeric,
    Roman,
    Letters,
    Symbols,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BibliographyConfig {
    pub enabled: bool,
    pub style: String, // "apa", "mla", "chicago", "ieee"
    pub title: String,
    pub sort_by: BibliographySort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BibliographySort {
    Author,
    Title,
    Year,
    Citation,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output: OutputConfig {
                format: "pdf".to_string(),
                quality: Some(1.0),
                compression: Some(true),
            },
            page: PageConfig {
                size: PageSize {
                    preset: Some("A4".to_string()),
                    width: None,
                    height: None,
                },
                margins: Margins {
                    top: "1in".to_string(),
                    right: "1in".to_string(),
                    bottom: "1in".to_string(),
                    left: "1in".to_string(),
                },
                orientation: Orientation::Portrait,
                header: None,
                footer: None,
                page_numbers: Some(PageNumberConfig {
                    enabled: false,
                    format: "Page {page} of {total}".to_string(),
                    position: PageNumberPosition::Footer,
                    start_from: Some(1),
                }),
            },
            theme: ThemeConfig {
                css_file: None,
                built_in: Some("default".to_string()),
                custom_styles: None,
            },
            fonts: FontConfig {
                family: Some("Arial, sans-serif".to_string()),
                size: Some("12pt".to_string()),
                line_height: Some(1.6),
                custom_fonts: None,
            },
            toc: TocConfig {
                enabled: true,
                title: "Table of Contents".to_string(),
                max_depth: 3,
                page_numbers: true,
                links: true,
                style: TocStyle::Indented,
            },
            images: ImageConfig {
                optimization: true,
                max_width: Some(800),
                max_height: Some(600),
                quality: Some(85),
                format: None, // Keep original format
            },
            code: CodeConfig {
                line_numbers: false,
                highlight_theme: "github".to_string(),
                word_wrap: true,
                show_language: true,
                copy_button: false,
            },
            references: ReferenceConfig {
                footnotes: FootnoteConfig {
                    enabled: true,
                    style: FootnoteStyle::Bottom,
                    numbering: FootnoteNumbering::Numeric,
                },
                bibliography: BibliographyConfig {
                    enabled: false,
                    style: "apa".to_string(),
                    title: "References".to_string(),
                    sort_by: BibliographySort::Author,
                },
                cross_references: true,
            },
        }
    }
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        // Open file with read access and acquire shared lock
        let file = fs::File::open(path)
            .with_context(|| format!("Failed to open config file: {}", path.display()))?;
        
        // Acquire shared lock for reading
        fs2::FileExt::lock_shared(&file)
            .with_context(|| format!("Failed to acquire read lock on config file: {}", path.display()))?;
        
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        // Lock is automatically released when file goes out of scope

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => {
                toml::from_str(&content)
                    .with_context(|| format!("Failed to parse TOML config: {}", path.display()))
            }
            Some("yaml") | Some("yml") => {
                serde_yaml::from_str(&content)
                    .with_context(|| format!("Failed to parse YAML config: {}", path.display()))
            }
            Some("json") => {
                serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse JSON config: {}", path.display()))
            }
            _ => anyhow::bail!(
                "Unsupported config file format. Use .toml, .yaml, .yml, or .json: {}",
                path.display()
            ),
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        
        let content = match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => toml::to_string_pretty(self)
                .with_context(|| "Failed to serialize config to TOML")?,
            Some("yaml") | Some("yml") => serde_yaml::to_string(self)
                .with_context(|| "Failed to serialize config to YAML")?,
            Some("json") => serde_json::to_string_pretty(self)
                .with_context(|| "Failed to serialize config to JSON")?,
            _ => anyhow::bail!(
                "Unsupported config file format. Use .toml, .yaml, .yml, or .json: {}",
                path.display()
            ),
        };

        // Use atomic write with file locking
        self.write_config_atomically(path, &content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }
    
    /// Atomically writes config content to file with proper locking
    fn write_config_atomically(&self, path: &Path, content: &str) -> Result<()> {
        use std::io::Write;
        
        // Create a temporary file in the same directory
        let temp_path = path.with_extension(format!("{}.tmp", 
            path.extension().and_then(|s| s.to_str()).unwrap_or("toml")));
        
        // Create and lock the temporary file
        let mut temp_file = fs::File::create(&temp_path)
            .with_context(|| format!("Failed to create temporary config file: {}", temp_path.display()))?;
        
        // Acquire exclusive lock on temporary file
        fs2::FileExt::lock_exclusive(&temp_file)
            .with_context(|| format!("Failed to acquire write lock on temporary config file: {}", temp_path.display()))?;
        
        // Write content to temporary file
        temp_file.write_all(content.as_bytes())
            .with_context(|| format!("Failed to write to temporary config file: {}", temp_path.display()))?;
        
        // Ensure data is written to disk
        temp_file.sync_all()
            .with_context(|| format!("Failed to sync temporary config file: {}", temp_path.display()))?;
        
        // Release lock before moving
        drop(temp_file);
        
        // Atomically replace the original file
        fs::rename(&temp_path, path)
            .with_context(|| format!("Failed to move temporary config file to final location: {}", path.display()))?;
        
        Ok(())
    }

    pub fn find_config_file() -> Option<PathBuf> {
        let possible_names = [
            "papercraft.toml",
            "papercraft.yaml",
            "papercraft.yml", 
            "papercraft.json",
            ".papercraft.toml",
            ".papercraft.yaml",
            ".papercraft.yml",
            ".papercraft.json",
            // Legacy names for backward compatibility
            "md-to-pdf.toml",
            "md-to-pdf.yaml",
            "md-to-pdf.yml",
            "md-to-pdf.json",
        ];

        // Check current directory first
        for name in &possible_names {
            let path = PathBuf::from(name);
            if path.exists() {
                return Some(path);
            }
        }

        // Check home directory
        if let Some(home_dir) = dirs::home_dir() {
            for name in &possible_names {
                let path = home_dir.join(name);
                if path.exists() {
                    return Some(path);
                }
            }
        }

        None
    }

    pub fn load_or_default() -> Result<Self> {
        if let Some(config_path) = Self::find_config_file() {
            Self::load_from_file(config_path)
        } else {
            Ok(Self::default())
        }
    }

    pub fn get_page_size_css(&self) -> String {
        match &self.page.size.preset {
            Some(preset) => match preset.to_uppercase().as_str() {
                "A4" => "@page { size: A4; }".to_string(),
                "LETTER" => "@page { size: Letter; }".to_string(),
                "LEGAL" => "@page { size: Legal; }".to_string(),
                "A3" => "@page { size: A3; }".to_string(),
                "A5" => "@page { size: A5; }".to_string(),
                _ => "@page { size: A4; }".to_string(), // Default fallback
            },
            None => {
                if let (Some(width), Some(height)) = (&self.page.size.width, &self.page.size.height) {
                    format!("@page {{ size: {width} {height}; }}")
                } else {
                    "@page { size: A4; }".to_string()
                }
            }
        }
    }

    pub fn get_margins_css(&self) -> String {
        format!(
            "@page {{ margin: {} {} {} {}; }}",
            self.page.margins.top,
            self.page.margins.right,
            self.page.margins.bottom,
            self.page.margins.left
        )
    }

    pub fn get_font_css(&self) -> String {
        let mut css = String::new();
        
        if let Some(family) = &self.fonts.family {
            css.push_str(&format!("body {{ font-family: {family}; }}\n"));
        }
        
        if let Some(size) = &self.fonts.size {
            css.push_str(&format!("body {{ font-size: {size}; }}\n"));
        }
        
        if let Some(line_height) = self.fonts.line_height {
            css.push_str(&format!("body {{ line-height: {line_height}; }}\n"));
        }

        // Add custom font declarations
        if let Some(custom_fonts) = &self.fonts.custom_fonts {
            for font in custom_fonts {
                let font_name = &font.name;
                let font_path = font.path.display();
                css.push_str(&format!(
                    "@font-face {{\n  font-family: '{font_name}';\n  src: url('{font_path}');\n"
                ));
                
                if let Some(weight) = &font.weight {
                    css.push_str(&format!("  font-weight: {weight};\n"));
                }
                
                if let Some(style) = &font.style {
                    css.push_str(&format!("  font-style: {style};\n"));
                }
                
                css.push_str("}\n");
            }
        }

        css
    }
}

// Add the dirs dependency to Cargo.toml if not present