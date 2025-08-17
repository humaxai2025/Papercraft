use anyhow::{Result, Context};
use std::io::{self, Write};
use std::path::PathBuf;
use crate::config::Config;
use crate::themes::ThemeManager;
use crate::logger::Logger;

pub struct ConfigWizard {
    theme_manager: ThemeManager,
}

impl ConfigWizard {
    pub fn new() -> Self {
        Self {
            theme_manager: ThemeManager::new(),
        }
    }
    
    pub fn run_interactive_setup(&self) -> Result<Config> {
        Logger::info("🎨 Welcome to PaperCraft Configuration Wizard!");
        Logger::info("This wizard will help you create a personalized configuration.");
        Logger::info("");
        
        let mut config = Config::default();
        
        // Theme selection
        self.configure_theme(&mut config)?;
        
        // Page settings
        self.configure_page_settings(&mut config)?;
        
        // Font settings
        self.configure_fonts(&mut config)?;
        
        // Advanced features
        self.configure_advanced_features(&mut config)?;
        
        // Output settings
        self.configure_output_settings(&mut config)?;
        
        Logger::success("Configuration completed!");
        Ok(config)
    }
    
    fn configure_theme(&self, config: &mut Config) -> Result<()> {
        Logger::step(1, 5, "Theme Selection");
        Logger::info("Choose a theme for your documents:");
        
        let themes = self.theme_manager.list_built_in_themes();
        for (i, theme) in themes.iter().enumerate() {
            println!("  {}. {}", i + 1, theme);
        }
        println!("  {}. Custom CSS file", themes.len() + 1);
        
        loop {
            let choice = self.prompt_input(&format!("Enter your choice (1-{}):", themes.len() + 1))?;
            
            if let Ok(num) = choice.parse::<usize>() {
                if num >= 1 && num <= themes.len() {
                    config.theme.built_in = Some(themes[num - 1].clone());
                    Logger::success(&format!("Selected theme: {}", themes[num - 1]));
                    break;
                } else if num == themes.len() + 1 {
                    let css_path = self.prompt_input("Enter path to custom CSS file:")?;
                    config.theme.css_file = Some(PathBuf::from(css_path));
                    Logger::success("Custom CSS theme configured");
                    break;
                } else {
                    Logger::warning("Invalid choice. Please try again.");
                }
            } else {
                Logger::warning("Please enter a number.");
            }
        }
        
        println!();
        Ok(())
    }
    
    fn configure_page_settings(&self, config: &mut Config) -> Result<()> {
        Logger::step(2, 5, "Page Settings");
        
        // Paper size
        Logger::info("Select paper size:");
        let sizes = ["A4", "Letter", "Legal", "A3", "A5"];
        for (i, size) in sizes.iter().enumerate() {
            println!("  {}. {}", i + 1, size);
        }
        
        loop {
            let choice = self.prompt_input(&format!("Enter your choice (1-{}) [default: A4]:", sizes.len()))?;
            
            if choice.is_empty() {
                config.page.size.preset = Some("A4".to_string());
                break;
            }
            
            if let Ok(num) = choice.parse::<usize>() {
                if num >= 1 && num <= sizes.len() {
                    config.page.size.preset = Some(sizes[num - 1].to_string());
                    Logger::success(&format!("Selected paper size: {}", sizes[num - 1]));
                    break;
                } else {
                    Logger::warning("Invalid choice. Please try again.");
                }
            } else {
                Logger::warning("Please enter a number.");
            }
        }
        
        // Orientation
        if self.prompt_yes_no("Use landscape orientation? [y/N]:", false)? {
            config.page.orientation = crate::config::Orientation::Landscape;
            Logger::success("Set orientation to landscape");
        }
        
        // Margins
        let margins = self.prompt_input("Enter margins (e.g., '1in', '20mm') [default: 1in]:")?;
        if !margins.is_empty() {
            config.page.margins.top = margins.clone();
            config.page.margins.right = margins.clone();
            config.page.margins.bottom = margins.clone();
            config.page.margins.left = margins;
            Logger::success("Custom margins configured");
        }
        
        println!();
        Ok(())
    }
    
    fn configure_fonts(&self, config: &mut Config) -> Result<()> {
        Logger::step(3, 5, "Font Settings");
        
        let font_family = self.prompt_input("Enter font family [default: system default]:")?;
        if !font_family.is_empty() {
            config.fonts.family = Some(font_family.clone());
            Logger::success(&format!("Set font family: {}", font_family));
        }
        
        let font_size = self.prompt_input("Enter font size (e.g., '12pt', '14px') [default: system default]:")?;
        if !font_size.is_empty() {
            config.fonts.size = Some(font_size.clone());
            Logger::success(&format!("Set font size: {}", font_size));
        }
        
        println!();
        Ok(())
    }
    
    fn configure_advanced_features(&self, config: &mut Config) -> Result<()> {
        Logger::step(4, 5, "Advanced Features");
        
        if self.prompt_yes_no("Enable table of contents? [Y/n]:", true)? {
            config.toc.enabled = true;
            Logger::success("Table of contents enabled");
        }
        
        if self.prompt_yes_no("Enable page numbers? [Y/n]:", true)? {
            config.page.page_numbers = Some(crate::config::PageNumberConfig {
                enabled: true,
                format: "Page {page} of {total}".to_string(),
                position: crate::config::PageNumberPosition::Footer,
                start_from: Some(1),
            });
            Logger::success("Page numbers enabled");
        }
        
        if self.prompt_yes_no("Enable code syntax highlighting with line numbers? [Y/n]:", true)? {
            config.code.line_numbers = true;
            config.code.highlight_theme = "Solarized (dark)".to_string();
            Logger::success("Code highlighting and line numbers enabled");
        }
        
        if self.prompt_yes_no("Enable image optimization? [Y/n]:", true)? {
            config.images.optimization = true;
            config.images.max_width = Some(800);
            Logger::success("Image optimization enabled");
        }
        
        if self.prompt_yes_no("Enable footnotes support? [y/N]:", false)? {
            config.references.footnotes.enabled = true;
            Logger::success("Footnotes support enabled");
        }
        
        println!();
        Ok(())
    }
    
    fn configure_output_settings(&self, config: &mut Config) -> Result<()> {
        Logger::step(5, 5, "Output Settings");
        
        if self.prompt_yes_no("Set up custom header? [y/N]:", false)? {
            let header = self.prompt_input("Enter header template (HTML):")?;
            if !header.is_empty() {
                config.page.header = Some(crate::config::HeaderFooterConfig {
                    enabled: true,
                    template: header,
                    height: Some("1cm".to_string()),
                    font_size: Some("10px".to_string()),
                });
                Logger::success("Custom header configured");
            }
        }
        
        if self.prompt_yes_no("Set up custom footer? [y/N]:", false)? {
            let footer = self.prompt_input("Enter footer template (HTML):")?;
            if !footer.is_empty() {
                config.page.footer = Some(crate::config::HeaderFooterConfig {
                    enabled: true,
                    template: footer,
                    height: Some("1cm".to_string()),
                    font_size: Some("10px".to_string()),
                });
                Logger::success("Custom footer configured");
            }
        }
        
        println!();
        Ok(())
    }
    
    fn prompt_input(&self, prompt: &str) -> Result<String> {
        print!("❓ {}", prompt);
        print!(" ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }
    
    fn prompt_yes_no(&self, prompt: &str, default: bool) -> Result<bool> {
        loop {
            let input = self.prompt_input(prompt)?;
            
            if input.is_empty() {
                return Ok(default);
            }
            
            match input.to_lowercase().as_str() {
                "y" | "yes" | "true" | "1" => return Ok(true),
                "n" | "no" | "false" | "0" => return Ok(false),
                _ => Logger::warning("Please enter y/yes or n/no"),
            }
        }
    }
    
    pub fn save_config_with_wizard(&self, config: &Config, suggested_path: Option<PathBuf>) -> Result<PathBuf> {
        let default_path = suggested_path.unwrap_or_else(|| PathBuf::from("papercraft-config.toml"));
        
        Logger::info("💾 Save Configuration");
        let path_str = self.prompt_input(&format!(
            "Enter config file path [default: {}]:",
            default_path.display()
        ))?;
        
        let config_path = if path_str.is_empty() {
            default_path
        } else {
            PathBuf::from(path_str)
        };
        
        // Show preview
        if self.prompt_yes_no("Show configuration preview? [Y/n]:", true)? {
            println!("\n📋 Configuration Preview:");
            println!("{}", "─".repeat(50));
            self.print_config_summary(config);
            println!("{}", "─".repeat(50));
        }
        
        if self.prompt_yes_no("Save this configuration? [Y/n]:", true)? {
            config.save_to_file(&config_path)
                .with_context(|| format!("Failed to save configuration to {}", config_path.display()))?;
            
            Logger::success(&format!("Configuration saved to: {}", config_path.display()));
            
            // Offer to create a sample document
            if self.prompt_yes_no("Create a sample markdown document to test? [Y/n]:", true)? {
                self.create_sample_document(&config_path.parent().unwrap_or(&PathBuf::from(".")))?;
            }
            
            Ok(config_path)
        } else {
            Err(anyhow::anyhow!("Configuration not saved"))
        }
    }
    
    fn print_config_summary(&self, config: &Config) {
        println!("🎨 Theme: {}", 
            config.theme.built_in.as_deref()
                .or(config.theme.css_file.as_ref().and_then(|p| p.file_name()).and_then(|n| n.to_str()))
                .unwrap_or("default"));
        
        println!("📄 Paper: {} ({})", 
            config.page.size.preset.as_deref().unwrap_or("A4"),
            match config.page.orientation {
                crate::config::Orientation::Portrait => "Portrait",
                crate::config::Orientation::Landscape => "Landscape",
            });
        
        if let Some(ref family) = config.fonts.family {
            println!("🔤 Font: {}", family);
        }
        
        println!("📚 Table of Contents: {}", if config.toc.enabled { "✓" } else { "✗" });
        println!("📝 Page Numbers: {}", 
            if config.page.page_numbers.as_ref().map_or(false, |p| p.enabled) { "✓" } else { "✗" });
        println!("💻 Code Line Numbers: {}", if config.code.line_numbers { "✓" } else { "✗" });
        println!("🖼️  Image Optimization: {}", if config.images.optimization { "✓" } else { "✗" });
    }
    
    fn create_sample_document(&self, dir: &std::path::Path) -> Result<()> {
        let sample_path = dir.join("sample-document.md");
        
        let sample_content = r#"# Sample PaperCraft Document

This is a sample document to test your PaperCraft configuration.

## Features

PaperCraft supports many markdown features:

- **Bold text** and *italic text*
- `Inline code` and code blocks
- Links like [this one](https://example.com)
- Images: ![GitHub Logo](https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png)

## Code Example

```rust
fn main() {
    println!("Hello, PaperCraft!");
}
```

## Table

| Feature | Status |
|---------|--------|
| Themes | ✓ |
| TOC | ✓ |
| Code Highlighting | ✓ |

## Conclusion

Your PaperCraft configuration is ready to use!

To convert this document to PDF, run:
```bash
papercraft -i sample-document.md -o sample-document.pdf -c your-config.toml
```
"#;
        
        std::fs::write(&sample_path, sample_content)
            .with_context(|| format!("Failed to create sample document: {}", sample_path.display()))?;
        
        Logger::success(&format!("Sample document created: {}", sample_path.display()));
        Ok(())
    }
}

#[allow(dead_code)]
pub fn run_first_time_setup() -> Result<()> {
    Logger::info("🎉 Welcome to PaperCraft!");
    Logger::info("It looks like this is your first time using PaperCraft.");
    Logger::info("");
    
    let wizard = ConfigWizard::new();
    
    if wizard.prompt_yes_no("Would you like to run the configuration wizard? [Y/n]:", true)? {
        let config = wizard.run_interactive_setup()?;
        let config_path = wizard.save_config_with_wizard(&config, Some(PathBuf::from("papercraft.toml")))?;
        
        Logger::info("");
        Logger::success("🎉 Setup complete!");
        Logger::info(&format!("Your configuration is saved at: {}", config_path.display()));
        Logger::info("You can now convert markdown files to PDF using PaperCraft.");
        Logger::info("");
        Logger::info("Quick start:");
        Logger::info(&format!("  papercraft -i input.md -o output.pdf -c {}", config_path.display()));
        Logger::info("");
        Logger::info("For more options, run: papercraft --help");
    } else {
        Logger::info("Skipping configuration wizard.");
        Logger::info("You can run it later with: papercraft --setup-wizard");
    }
    
    Ok(())
}