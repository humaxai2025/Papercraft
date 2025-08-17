use anyhow::{Result, Context};
use std::path::Path;
use std::fs;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref MARKDOWN_LINK_REGEX: Regex = Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").unwrap();
    static ref MARKDOWN_IMAGE_REGEX: Regex = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();
    static ref MARKDOWN_HEADING_REGEX: Regex = Regex::new(r"^(#{1,6})\s+(.+)$").unwrap();
    static ref MARKDOWN_CODE_BLOCK_REGEX: Regex = Regex::new(r"```([a-zA-Z]*)\n([\s\S]*?)\n```").unwrap();
    static ref MARKDOWN_TABLE_REGEX: Regex = Regex::new(r"^\|(.+)\|$").unwrap();
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub line: usize,
    pub column: Option<usize>,
    pub severity: IssueSeverity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub issues: Vec<ValidationIssue>,
    pub stats: ValidationStats,
}

#[derive(Debug)]
pub struct ValidationStats {
    pub total_lines: usize,
    pub headings: usize,
    pub links: usize,
    pub images: usize,
    pub code_blocks: usize,
    pub tables: usize,
    pub errors: usize,
    pub warnings: usize,
}

pub struct MarkdownValidator {
    check_links: bool,
    check_images: bool,
    check_tables: bool,
    check_headings: bool,
    base_path: Option<std::path::PathBuf>,
}

impl MarkdownValidator {
    pub fn new() -> Self {
        Self {
            check_links: true,
            check_images: true,
            check_tables: true,
            check_headings: true,
            base_path: None,
        }
    }
    
    pub fn with_base_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.base_path = Some(path.as_ref().to_path_buf());
        self
    }
    
    pub fn validate_file<P: AsRef<Path>>(&self, file_path: P) -> Result<ValidationResult> {
        let path = file_path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        self.validate_content(&content)
    }
    
    pub fn validate_content(&self, content: &str) -> Result<ValidationResult> {
        let mut issues = Vec::new();
        let mut stats = ValidationStats {
            total_lines: 0,
            headings: 0,
            links: 0,
            images: 0,
            code_blocks: 0,
            tables: 0,
            errors: 0,
            warnings: 0,
        };
        
        let lines: Vec<&str> = content.lines().collect();
        stats.total_lines = lines.len();
        
        // Track state for multi-line constructs
        let mut in_code_block = false;
        let mut table_started = false;
        let mut table_header_found = false;
        
        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;
            
            // Check for code blocks
            if line.trim_start().starts_with("```") {
                if !in_code_block {
                    in_code_block = true;
                    stats.code_blocks += 1;
                    self.validate_code_block_start(line, line_num, &mut issues);
                } else {
                    in_code_block = false;
                }
                continue;
            }
            
            // Skip validation inside code blocks
            if in_code_block {
                continue;
            }
            
            // Check headings
            if self.check_headings {
                self.validate_headings(line, line_num, &mut issues, &mut stats);
            }
            
            // Check links
            if self.check_links {
                self.validate_links(line, line_num, &mut issues, &mut stats);
            }
            
            // Check images
            if self.check_images {
                self.validate_images(line, line_num, &mut issues, &mut stats);
            }
            
            // Check tables
            if self.check_tables {
                self.validate_tables(line, line_num, &mut issues, &mut stats, &mut table_started, &mut table_header_found);
            }
            
            // Check for common markdown issues
            self.validate_common_issues(line, line_num, &mut issues);
        }
        
        // Count severity levels
        for issue in &issues {
            match issue.severity {
                IssueSeverity::Error => stats.errors += 1,
                IssueSeverity::Warning => stats.warnings += 1,
                IssueSeverity::Info => {},
            }
        }
        
        Ok(ValidationResult { issues, stats })
    }
    
    fn validate_headings(&self, line: &str, line_num: usize, issues: &mut Vec<ValidationIssue>, stats: &mut ValidationStats) {
        if let Some(caps) = MARKDOWN_HEADING_REGEX.captures(line) {
            stats.headings += 1;
            let level = caps[1].len();
            let text = &caps[2];
            
            // Check for empty headings
            if text.trim().is_empty() {
                issues.push(ValidationIssue {
                    line: line_num,
                    column: Some(caps[1].len() + 1),
                    severity: IssueSeverity::Warning,
                    message: "Empty heading found".to_string(),
                    suggestion: Some("Add meaningful heading text".to_string()),
                });
            }
            
            // Check for heading level jumps
            if level > 3 {
                issues.push(ValidationIssue {
                    line: line_num,
                    column: Some(1),
                    severity: IssueSeverity::Info,
                    message: format!("Deep heading level (h{})", level),
                    suggestion: Some("Consider restructuring with fewer heading levels".to_string()),
                });
            }
        }
    }
    
    fn validate_links(&self, line: &str, line_num: usize, issues: &mut Vec<ValidationIssue>, stats: &mut ValidationStats) {
        for caps in MARKDOWN_LINK_REGEX.captures_iter(line) {
            stats.links += 1;
            let link_text = &caps[1];
            let url = &caps[2];
            
            // Check for empty link text
            if link_text.trim().is_empty() {
                issues.push(ValidationIssue {
                    line: line_num,
                    column: Some(caps.get(1).unwrap().start()),
                    severity: IssueSeverity::Warning,
                    message: "Link with empty text".to_string(),
                    suggestion: Some("Add descriptive link text".to_string()),
                });
            }
            
            // Check for suspicious URLs
            if url.trim().is_empty() {
                issues.push(ValidationIssue {
                    line: line_num,
                    column: Some(caps.get(2).unwrap().start()),
                    severity: IssueSeverity::Error,
                    message: "Link with empty URL".to_string(),
                    suggestion: Some("Add a valid URL".to_string()),
                });
            } else if url == "#" {
                issues.push(ValidationIssue {
                    line: line_num,
                    column: Some(caps.get(2).unwrap().start()),
                    severity: IssueSeverity::Warning,
                    message: "Placeholder link found".to_string(),
                    suggestion: Some("Replace with actual URL".to_string()),
                });
            }
            
            // Check for local file links if base path is set
            if let Some(ref base_path) = self.base_path {
                if !url.starts_with("http") && !url.starts_with("mailto:") && !url.starts_with("#") {
                    let file_path = base_path.join(url);
                    if !file_path.exists() {
                        issues.push(ValidationIssue {
                            line: line_num,
                            column: Some(caps.get(2).unwrap().start()),
                            severity: IssueSeverity::Warning,
                            message: format!("Local file link not found: {}", url),
                            suggestion: Some("Check file path or create the referenced file".to_string()),
                        });
                    }
                }
            }
        }
    }
    
    fn validate_images(&self, line: &str, line_num: usize, issues: &mut Vec<ValidationIssue>, stats: &mut ValidationStats) {
        for caps in MARKDOWN_IMAGE_REGEX.captures_iter(line) {
            stats.images += 1;
            let alt_text = &caps[1];
            let src = &caps[2];
            
            // Check for missing alt text
            if alt_text.trim().is_empty() {
                issues.push(ValidationIssue {
                    line: line_num,
                    column: Some(caps.get(1).unwrap().start()),
                    severity: IssueSeverity::Warning,
                    message: "Image missing alt text".to_string(),
                    suggestion: Some("Add descriptive alt text for accessibility".to_string()),
                });
            }
            
            // Check for empty image source
            if src.trim().is_empty() {
                issues.push(ValidationIssue {
                    line: line_num,
                    column: Some(caps.get(2).unwrap().start()),
                    severity: IssueSeverity::Error,
                    message: "Image with empty source".to_string(),
                    suggestion: Some("Add a valid image path or URL".to_string()),
                });
            }
            
            // Check for local image files if base path is set
            if let Some(ref base_path) = self.base_path {
                if !src.starts_with("http") && !src.starts_with("data:") {
                    let image_path = base_path.join(src);
                    if !image_path.exists() {
                        issues.push(ValidationIssue {
                            line: line_num,
                            column: Some(caps.get(2).unwrap().start()),
                            severity: IssueSeverity::Warning,
                            message: format!("Image file not found: {}", src),
                            suggestion: Some("Check image path or add the referenced image".to_string()),
                        });
                    }
                }
            }
        }
    }
    
    fn validate_tables(&self, line: &str, line_num: usize, issues: &mut Vec<ValidationIssue>, stats: &mut ValidationStats, table_started: &mut bool, table_header_found: &mut bool) {
        if MARKDOWN_TABLE_REGEX.is_match(line) {
            if !*table_started {
                *table_started = true;
                *table_header_found = false;
                stats.tables += 1;
            }
            
            // Check for table separator row
            if line.contains("---") || line.contains("===") {
                *table_header_found = true;
            }
            
            // Count columns and check consistency
            let columns = line.split('|').filter(|s| !s.trim().is_empty()).count();
            if columns < 2 {
                issues.push(ValidationIssue {
                    line: line_num,
                    column: Some(1),
                    severity: IssueSeverity::Warning,
                    message: "Table row with insufficient columns".to_string(),
                    suggestion: Some("Ensure table rows have at least 2 columns".to_string()),
                });
            }
        } else if *table_started {
            // End of table
            if !*table_header_found {
                issues.push(ValidationIssue {
                    line: line_num - 1,
                    column: Some(1),
                    severity: IssueSeverity::Warning,
                    message: "Table missing header separator row".to_string(),
                    suggestion: Some("Add a row with --- separators after the header".to_string()),
                });
            }
            *table_started = false;
            *table_header_found = false;
        }
    }
    
    fn validate_code_block_start(&self, line: &str, line_num: usize, issues: &mut Vec<ValidationIssue>) {
        let trimmed = line.trim_start();
        if trimmed.len() > 3 {
            let language = &trimmed[3..].trim();
            if language.is_empty() {
                issues.push(ValidationIssue {
                    line: line_num,
                    column: Some(line.len()),
                    severity: IssueSeverity::Info,
                    message: "Code block without language specification".to_string(),
                    suggestion: Some("Add language for syntax highlighting (e.g., ```rust)".to_string()),
                });
            }
        }
    }
    
    fn validate_common_issues(&self, line: &str, line_num: usize, issues: &mut Vec<ValidationIssue>) {
        // Check for trailing whitespace
        if line.ends_with(' ') || line.ends_with('\t') {
            issues.push(ValidationIssue {
                line: line_num,
                column: Some(line.len()),
                severity: IssueSeverity::Info,
                message: "Line has trailing whitespace".to_string(),
                suggestion: Some("Remove trailing spaces".to_string()),
            });
        }
        
        // Check for very long lines
        if line.len() > 120 {
            issues.push(ValidationIssue {
                line: line_num,
                column: Some(120),
                severity: IssueSeverity::Info,
                message: format!("Long line ({} characters)", line.len()),
                suggestion: Some("Consider breaking long lines for readability".to_string()),
            });
        }
        
        // Check for multiple consecutive blank lines
        if line.trim().is_empty() {
            // This would need more context to implement properly
        }
    }
}

impl ValidationResult {
    pub fn has_errors(&self) -> bool {
        self.stats.errors > 0
    }
    
    #[allow(dead_code)]
    pub fn has_warnings(&self) -> bool {
        self.stats.warnings > 0
    }
    
    pub fn is_valid(&self) -> bool {
        !self.has_errors()
    }
    
    pub fn print_summary(&self) {
        println!("📊 Validation Summary:");
        println!("  📄 Total lines: {}", self.stats.total_lines);
        println!("  📝 Headings: {}", self.stats.headings);
        println!("  🔗 Links: {}", self.stats.links);
        println!("  🖼️  Images: {}", self.stats.images);
        println!("  💻 Code blocks: {}", self.stats.code_blocks);
        println!("  📋 Tables: {}", self.stats.tables);
        println!("  ❌ Errors: {}", self.stats.errors);
        println!("  ⚠️  Warnings: {}", self.stats.warnings);
    }
    
    pub fn print_issues(&self, show_info: bool) {
        if self.issues.is_empty() {
            println!("✓ No validation issues found!");
            return;
        }
        
        for issue in &self.issues {
            if !show_info && issue.severity == IssueSeverity::Info {
                continue;
            }
            
            let icon = match issue.severity {
                IssueSeverity::Error => "❌",
                IssueSeverity::Warning => "⚠️ ",
                IssueSeverity::Info => "ℹ️ ",
            };
            
            if let Some(col) = issue.column {
                println!("{}  Line {}, Column {}: {}", icon, issue.line, col, issue.message);
            } else {
                println!("{}  Line {}: {}", icon, issue.line, issue.message);
            }
            
            if let Some(ref suggestion) = issue.suggestion {
                println!("     💡 {}", suggestion);
            }
        }
    }
}