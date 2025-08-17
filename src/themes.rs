use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// All built-in themes are embedded at compile time for security and portability
// Users cannot modify these themes as they're baked into the binary
// Only external custom CSS files can be loaded via the css_file configuration option
const DEFAULT_THEME: &str = include_str!("themes/default.css");
const DARK_THEME: &str = include_str!("themes/dark.css");
const MINIMAL_THEME: &str = include_str!("themes/minimal.css");
const ACADEMIC_THEME: &str = include_str!("themes/academic.css");
const MODERN_THEME: &str = include_str!("themes/modern.css");

pub struct ThemeManager {
    built_in_themes: HashMap<String, &'static str>,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut built_in_themes = HashMap::new();
        
        // All themes are baked into the binary at compile time
        built_in_themes.insert("default".to_string(), DEFAULT_THEME);
        built_in_themes.insert("dark".to_string(), DARK_THEME);
        built_in_themes.insert("minimal".to_string(), MINIMAL_THEME);
        built_in_themes.insert("academic".to_string(), ACADEMIC_THEME);
        built_in_themes.insert("modern".to_string(), MODERN_THEME);

        Self { built_in_themes }
    }

    pub fn get_theme(&self, theme_name: &str) -> Option<&str> {
        self.built_in_themes.get(theme_name).copied()
    }

    pub fn load_external_theme<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to load theme file: {}", path.as_ref().display()))
    }

    pub fn list_built_in_themes(&self) -> Vec<&String> {
        self.built_in_themes.keys().collect()
    }

    pub fn resolve_theme(&self, theme_name: Option<&str>, theme_file: Option<&Path>) -> Result<String> {
        match (theme_name, theme_file) {
            (_, Some(file)) => self.load_external_theme(file),
            (Some(name), None) => {
                self.get_theme(name)
                    .map(|s| s.to_string())
                    .ok_or_else(|| anyhow::anyhow!("Unknown built-in theme: {}", name))
            }
            (None, None) => Ok(self.get_theme("default").unwrap().to_string()),
        }
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}