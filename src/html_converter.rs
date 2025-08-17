use anyhow::{Context, Result};
use comrak::{markdown_to_html, ComrakOptions};
use headless_chrome::{types::PrintToPdfOptions, Browser, LaunchOptions};
use lazy_static::lazy_static;
// use scraper::{Html, Selector}; // Removed - now handled by AdvancedProcessor
use std::fs;
use std::path::Path;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::html::highlighted_html_for_string;
use regex::Regex;
use syntect::parsing::SyntaxSet;
use base64::{engine::general_purpose::STANDARD, Engine};
use crate::config::Config;
use crate::themes::ThemeManager;
use crate::advanced_processing::AdvancedProcessor;
use crate::image_optimization::ImageOptimizer;
use crate::advanced_styles::AdvancedStyles;
use crate::chrome_manager::ChromeManager;

// Struct for conversion options
#[derive(Clone)]
pub struct ConversionOptions {
    pub config: Config,
}

lazy_static! {
    // Load syntax and theme sets once
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
    static ref DEFAULT_THEME: &'static Theme = &THEME_SET.themes["Solarized (dark)"];
    // Regex to find code blocks
    static ref CODE_BLOCK_REGEX: Regex = Regex::new(r#"<pre><code class="language-(?P<lang>[^>]*)">(?P<code>[^<]*)</code></pre>"#).unwrap();
    // Regex to find inline and block math (with non-greedy matching and multiline support)
    static ref MATH_BLOCK_REGEX: Regex = Regex::new(r#"(?s)\$\$([^$]+?)\$\$"#).unwrap();
    static ref INLINE_MATH_REGEX: Regex = Regex::new(r#"\$([^$\n]+?)\$"#).unwrap();
}

// const DEFAULT_CSS: &str = include_str!("default-theme.css"); // Now handled by theme manager

pub struct HtmlToPdfConverter {
    theme_manager: ThemeManager,
    chrome_manager: ChromeManager,
}

impl HtmlToPdfConverter {
    pub fn new() -> Result<Self> {
        let chrome_manager = ChromeManager::new()
            .context("Failed to initialize Chrome manager")?;
        
        Ok(Self {
            theme_manager: ThemeManager::new(),
            chrome_manager,
        })
    }
    
    /// Check if Chrome is available or needs to be downloaded
    pub fn check_chrome_status(&self) -> Result<()> {
        if self.chrome_manager.is_chrome_available() {
            let version = self.chrome_manager.get_chrome_version()
                .unwrap_or_else(|_| "Unknown".to_string());
            crate::logger::Logger::verbose(&format!("Chrome available: {}", version));
            Ok(())
        } else {
            crate::logger::Logger::info("Chrome Headless Shell will be downloaded on first use");
            Ok(())
        }
    }

    pub fn convert_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: ConversionOptions,
    ) -> Result<()> {
        let markdown_content = fs::read_to_string(input_path)
            .with_context(|| format!("Failed to read file: {}", input_path.display()))?;

        let raw_html = self.markdown_to_html(&markdown_content)?;

        let final_html = self.enhance_html(&raw_html, &options, input_path)?;

        self.html_to_pdf(&final_html, output_path, &options)?;

        Ok(())
    }

    fn markdown_to_html(&self, markdown: &str) -> Result<String> {
        let mut options = ComrakOptions::default();
        options.extension.strikethrough = true;
        options.extension.table = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options.extension.superscript = true;
        options.extension.footnotes = true;
        options.extension.header_ids = Some("user-content-".to_string());
        options.render.unsafe_ = false; // Disable raw HTML for security
        // Note: This means we'll handle ToC and other features through processing pipeline
        
        Ok(markdown_to_html(markdown, &options))
    }

    fn enhance_html(&self, raw_html: &str, options: &ConversionOptions, input_path: &Path) -> Result<String> {
        // Process Mermaid blocks first (before syntax highlighting)
        let mut processed_html = self.prepare_mermaid_blocks(raw_html)?;

        // Process Math blocks
        processed_html = self.prepare_math_blocks(&processed_html)?;

        // Then perform syntax highlighting (excluding mermaid)
        processed_html = self.apply_syntax_highlighting_by_string(&processed_html)?;

        // Advanced processing for new features
        let mut advanced_processor = AdvancedProcessor::new(options.config.clone());
        processed_html = advanced_processor.process_document(&processed_html)?;

        // Image optimization
        let base_path = input_path.parent().unwrap_or(Path::new("."));
        let mut image_optimizer = ImageOptimizer::new(options.config.images.clone());
        processed_html = image_optimizer.process_images_in_html(&processed_html, base_path)?;

        // --- Final HTML Assembly ---
        let theme_css = self.get_theme_css(&options.config)?;
        let final_html = self.assemble_final_html(&processed_html, &theme_css, &options.config);

        Ok(final_html)
    }

    fn prepare_mermaid_blocks(&self, html: &str) -> Result<String> {
        // Use regex to replace mermaid code blocks directly
        let mermaid_regex = Regex::new(r#"(?s)<pre><code class="language-mermaid">(.*?)</code></pre>"#).unwrap();
        
        let result = mermaid_regex.replace_all(html, |caps: &regex::Captures| {
            let code = caps.get(1).map_or("", |m| m.as_str()).trim();
            format!("<div class=\"mermaid\">{code}</div>")
        });
        
        Ok(result.into_owned())
    }

    fn prepare_math_blocks(&self, html: &str) -> Result<String> {
        // Process block math first ($$...$$)
        let block_processed = MATH_BLOCK_REGEX.replace_all(html, |caps: &regex::Captures| {
            let content = caps.get(1).map_or("", |m| m.as_str()).trim();
            format!("<div class=\"math-display\">$${content}$$</div>")
        });
        
        // Then process inline math ($...$)
        let inline_processed = INLINE_MATH_REGEX.replace_all(&block_processed, |caps: &regex::Captures| {
            let content = caps.get(1).map_or("", |m| m.as_str()).trim();
            format!("<span class=\"math-inline\">${content}$</span>")
        });
        
        Ok(inline_processed.into_owned())
    }

    fn get_theme_css(&self, config: &Config) -> Result<String> {
        let base_css = match (&config.theme.css_file, &config.theme.built_in) {
            (Some(css_file), _) => self.theme_manager.load_external_theme(css_file)?,
            (None, Some(theme_name)) => self.theme_manager.resolve_theme(Some(theme_name), None)?,
            (None, None) => self.theme_manager.resolve_theme(Some("default"), None)?,
        };

        let mut combined_css = base_css;
        
        // Add page layout CSS
        combined_css.push_str("\n\n/* Page Layout */\n");
        combined_css.push_str(&config.get_page_size_css());
        combined_css.push('\n');
        combined_css.push_str(&config.get_margins_css());
        
        // Add font CSS
        combined_css.push_str("\n\n/* Fonts */\n");
        combined_css.push_str(&config.get_font_css());
        
        // Add advanced feature styles
        combined_css.push_str("\n\n/* Advanced Features */\n");
        combined_css.push_str(&AdvancedStyles::get_all_advanced_styles(config));

        // Add custom styles if any
        if let Some(custom_styles) = &config.theme.custom_styles {
            combined_css.push_str("\n\n/* Custom Styles */\n");
            for (selector, style) in custom_styles {
                combined_css.push_str(&format!("{selector} {{ {style} }}\n"));
            }
        }

        Ok(combined_css)
    }

    fn assemble_final_html(&self, body: &str, css: &str, _config: &Config) -> String {
        let render_timeout = self.calculate_js_timeout(body);
        
        format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Document</title>
    <style>{css}</style>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css">
    <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js"></script>
    <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/contrib/auto-render.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/mermaid@9.4.3/dist/mermaid.min.js"></script>
</head>
<body>
    {body}
    <script>
        window.addEventListener('load', function() {{
            // Initialize Mermaid with simple configuration
            mermaid.initialize({{ 
                startOnLoad: true,
                theme: 'default',
                securityLevel: 'loose'
            }});
            
            // Render math equations
            renderMathInElement(document.body, {{
                delimiters: [
                    {{left: "$$", right: "$$", display: true}},
                    {{left: "$", right: "$", display: false}}
                ],
                throwOnError: false
            }});

            // Signal that rendering is done after libraries have processed
            window.setTimeout(() => {{ 
                const sentinel = document.createElement('div');
                sentinel.id = 'render_done';
                document.body.appendChild(sentinel);
            }}, {render_timeout});
        }});
    </script>
</body>
</html>"#)
    }

    fn apply_syntax_highlighting_by_string(&self, html: &str) -> Result<String> {
        let result = CODE_BLOCK_REGEX.replace_all(html, |caps: &regex::Captures| {
            let lang = caps.name("lang").map_or("txt", |m| m.as_str());
            let code = caps.name("code").map_or("", |m| m.as_str());
            
            
            // Skip mermaid blocks - they should already be processed
            if lang == "mermaid" {
                return format!("<pre><code class=\"language-{lang}\">{code}</code></pre>");
            }
            
            // The code from regex can have escaped characters, we need to unescape it.
            let unescaped_code = html_escape::decode_html_entities(code).to_string();

            // Better syntax detection by language name
            let syntax = SYNTAX_SET.find_syntax_by_extension(lang)
                .or_else(|| SYNTAX_SET.find_syntax_by_name(lang))
                .or_else(|| match lang {
                    "rs" | "rust" => SYNTAX_SET.find_syntax_by_name("Rust"),
                    "py" | "python" => SYNTAX_SET.find_syntax_by_name("Python"),
                    "js" | "javascript" => SYNTAX_SET.find_syntax_by_name("JavaScript"),
                    "ts" | "typescript" => SYNTAX_SET.find_syntax_by_name("TypeScript"),
                    "c" => SYNTAX_SET.find_syntax_by_name("C"),
                    "cpp" | "c++" => SYNTAX_SET.find_syntax_by_name("C++"),
                    "java" => SYNTAX_SET.find_syntax_by_name("Java"),
                    "go" => SYNTAX_SET.find_syntax_by_name("Go"),
                    "bash" | "sh" | "shell" => {
                        // Try common bash/shell syntax names
                        SYNTAX_SET.find_syntax_by_extension("sh")
                            .or_else(|| SYNTAX_SET.find_syntax_by_name("Shell-Unix-Generic"))
                            .or_else(|| SYNTAX_SET.find_syntax_by_name("Bourne Again Shell (bash)"))
                            .or_else(|| SYNTAX_SET.find_syntax_by_name("Shell Script (Bash)"))
                            .or_else(|| SYNTAX_SET.find_syntax_by_name("Bash"))
                    },
                    "console" | "terminal" => SYNTAX_SET.find_syntax_by_extension("sh"),
                    "cmd" | "bat" | "batch" => SYNTAX_SET.find_syntax_by_name("Batch File"),
                    "ps1" | "powershell" => SYNTAX_SET.find_syntax_by_name("PowerShell"),
                    "zsh" => SYNTAX_SET.find_syntax_by_extension("sh"),
                    _ => None
                })
                .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
            
            highlighted_html_for_string(&unescaped_code, &SYNTAX_SET, syntax, &DEFAULT_THEME).unwrap_or_else(|_| "<pre>Error highlighting code</pre>".to_string())
        });
        Ok(result.into_owned())
    }
    
    // Old ToC generation method removed - now handled by AdvancedProcessor

    fn html_to_pdf(
        &self,
        html: &str,
        output_path: &Path,
        options: &ConversionOptions,
    ) -> Result<()> {
        // Use a closure with proper cleanup to ensure browser is dropped
        // Even if conversion fails, we want to ensure proper cleanup
        self.convert_with_browser(html, output_path, options)
    }
    
    fn convert_with_browser(
        &self,
        html: &str,
        output_path: &Path,
        options: &ConversionOptions,
    ) -> Result<()> {
        // Ensure Chrome is available (download if necessary)
        let chrome_path = self.chrome_manager.ensure_chrome()
            .context("Failed to ensure Chrome availability")?;
        
        // Launch Chrome with custom path
        let launch_options = LaunchOptions::default_builder()
            .path(Some(chrome_path))
            .headless(true)
            .build()
            .context("Failed to build Chrome launch options")?;
        
        let browser = Browser::new(launch_options)
            .context("Failed to launch headless Chrome")?;
        
        // Ensure browser cleanup on drop with a custom wrapper
        let _browser_guard = BrowserGuard::new(&browser);
        
        let tab = browser.new_tab()?;

        let encoded_html = STANDARD.encode(html);
        let data_url = format!("data:text/html;base64,{encoded_html}");

        tab.navigate_to(&data_url)?;
        
        // Calculate dynamic timeout based on content complexity
        let timeout_duration = self.calculate_render_timeout(html);
        let start_time = std::time::Instant::now();
        
        // Wait for render completion with dynamic timeout
        while start_time.elapsed() < timeout_duration {
            if tab.find_element("#render_done").is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        // Check if we actually found the element
        if tab.find_element("#render_done").is_err() {
            return Err(anyhow::anyhow!("Timeout waiting for page rendering to complete"));
        }

        let (header_template, footer_template) = self.build_header_footer_templates(&options.config);
        
        let pdf_options = PrintToPdfOptions {
            header_template,
            footer_template,
            print_background: Some(true),
            prefer_css_page_size: Some(true),
            landscape: Some(matches!(options.config.page.orientation, crate::config::Orientation::Landscape)),
            display_header_footer: Some(
                options.config.page.header.as_ref().map(|h| h.enabled).unwrap_or(false) ||
                options.config.page.footer.as_ref().map(|f| f.enabled).unwrap_or(false) ||
                options.config.page.page_numbers.as_ref().map(|p| p.enabled).unwrap_or(false)
            ),
            ..Default::default()
        };

        let pdf_data = tab.print_to_pdf(Some(pdf_options))?;
        fs::write(output_path, pdf_data)
            .with_context(|| format!("Failed to write PDF to {}", output_path.display()))?;

        // Browser will be properly cleaned up by BrowserGuard when it goes out of scope
        Ok(())
    }

    fn build_header_footer_templates(&self, config: &Config) -> (Option<String>, Option<String>) {
        let header_template = if let Some(header_config) = &config.page.header {
            if header_config.enabled {
                Some(format!(
                    r#"<div style="font-size: {}; height: {}; width: 100%; text-align: center;">{}</div>"#,
                    header_config.font_size.as_deref().unwrap_or("10px"),
                    header_config.height.as_deref().unwrap_or("1cm"),
                    header_config.template
                ))
            } else {
                None
            }
        } else {
            None
        };

        let footer_template = if let Some(footer_config) = &config.page.footer {
            if footer_config.enabled {
                Some(format!(
                    r#"<div style="font-size: {}; height: {}; width: 100%; text-align: center;">{}</div>"#,
                    footer_config.font_size.as_deref().unwrap_or("10px"),
                    footer_config.height.as_deref().unwrap_or("1cm"),
                    footer_config.template
                ))
            } else {
                None
            }
        } else if let Some(page_numbers) = &config.page.page_numbers {
            if page_numbers.enabled {
                let template = match page_numbers.position {
                    crate::config::PageNumberPosition::Footer => {
                        page_numbers.format
                            .replace("{page}", r#"<span class="pageNumber"></span>"#)
                            .replace("{total}", r#"<span class="totalPages"></span>"#)
                    },
                    crate::config::PageNumberPosition::Header => String::new(),
                };
                
                if !template.is_empty() {
                    Some(format!(
                        r#"<div style="font-size: 10px; height: 1cm; width: 100%; text-align: center;">{template}</div>"#
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // If page numbers are in header, build header template
        let header_template = if header_template.is_none() {
            if let Some(page_numbers) = &config.page.page_numbers {
                if page_numbers.enabled && matches!(page_numbers.position, crate::config::PageNumberPosition::Header) {
                    let template = page_numbers.format
                        .replace("{page}", r#"<span class="pageNumber"></span>"#)
                        .replace("{total}", r#"<span class="totalPages"></span>"#);
                    
                    Some(format!(
                        r#"<div style="font-size: 10px; height: 1cm; width: 100%; text-align: center;">{template}</div>"#
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            header_template
        };

        (header_template, footer_template)
    }
    
    /// Calculate dynamic render timeout based on content complexity
    fn calculate_render_timeout(&self, html: &str) -> std::time::Duration {
        let mut timeout_seconds = 5; // Base timeout
        
        // Add time for math rendering
        let math_count = html.matches("$$").count() + html.matches("$").count();
        timeout_seconds += (math_count / 10).max(0);
        
        // Add time for mermaid diagrams
        let mermaid_count = html.matches("class=\"mermaid\"").count();
        timeout_seconds += mermaid_count * 3;
        
        // Add time for images
        let image_count = html.matches("<img").count();
        timeout_seconds += (image_count / 5).max(0);
        
        // Add time for large documents
        let content_size_kb = html.len() / 1024;
        if content_size_kb > 100 {
            timeout_seconds += (content_size_kb / 100).min(10);
        }
        
        // Clamp between 5 and 60 seconds
        timeout_seconds = timeout_seconds.clamp(5, 60);
        
        std::time::Duration::from_secs(timeout_seconds as u64)
    }
    
    /// Calculate JavaScript timeout for rendering libraries
    fn calculate_js_timeout(&self, html: &str) -> u32 {
        let mut timeout_ms = 2000; // Base 2 seconds
        
        // Add time for math rendering
        let math_count = html.matches("$$").count() + html.matches("$").count();
        timeout_ms += (math_count * 100).min(3000);
        
        // Add time for mermaid diagrams
        let mermaid_count = html.matches("class=\"mermaid\"").count();
        timeout_ms += mermaid_count * 1500;
        
        // Clamp between 1 and 15 seconds
        timeout_ms.clamp(1000, 15000) as u32
    }
}

/// RAII guard to ensure proper browser cleanup
struct BrowserGuard<'a> {
    _browser: &'a Browser,
}

impl<'a> BrowserGuard<'a> {
    fn new(browser: &'a Browser) -> Self {
        Self { _browser: browser }
    }
}

impl<'a> Drop for BrowserGuard<'a> {
    fn drop(&mut self) {
        // Attempt to close the browser gracefully
        // Note: headless_chrome Browser implements Drop, but we can add extra cleanup here
        if self._browser.get_version().is_err() {
            // Browser is already closed or in error state
            // Could log this if we had logging available
        }
        // The actual browser cleanup happens in the Browser's Drop implementation
    }
}