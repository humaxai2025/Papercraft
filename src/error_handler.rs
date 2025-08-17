use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PapercraftError {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
    
    #[error("Invalid file format: {path}. Expected markdown (.md) file")]
    InvalidFileFormat { path: PathBuf },
    
    #[error("Permission denied accessing: {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("Configuration error in {file}: {message}")]
    #[allow(dead_code)]
    ConfigError { file: String, message: String },
    
    #[error("Conversion failed for {input} -> {output}: {reason}")]
    ConversionFailed {
        input: PathBuf,
        output: PathBuf,
        reason: String,
    },
    
    #[error("Chrome browser initialization failed: {reason}")]
    #[allow(dead_code)]
    BrowserInitFailed { reason: String },
    
    #[error("Memory limit exceeded processing file: {path}. File size: {size_mb}MB")]
    MemoryLimitExceeded { path: PathBuf, size_mb: u64 },
    
    #[error("Processing timeout for file: {path}. Timeout: {timeout_seconds}s")]
    #[allow(dead_code)]
    ProcessingTimeout { path: PathBuf, timeout_seconds: u64 },
    
    #[error("Disk space insufficient. Required: {required_mb}MB, Available: {available_mb}MB")]
    #[allow(dead_code)]
    InsufficientDiskSpace { required_mb: u64, available_mb: u64 },
    
    #[error("Template error: {message}")]
    #[allow(dead_code)]
    TemplateError { message: String },
    
    #[error("Image processing error for {path}: {reason}")]
    #[allow(dead_code)]
    ImageProcessingError { path: PathBuf, reason: String },
    
    #[error("Path traversal attempt detected: {path}")]
    PathTraversalAttempt { path: PathBuf },
}

pub struct ErrorReporter {
    verbose: bool,
}

impl ErrorReporter {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
    
    pub fn report_error(&self, error: &PapercraftError) {
        eprintln!("❌ Error: {error}");
        
        if self.verbose {
            self.print_detailed_error(error);
        }
        
        self.print_suggestions(error);
    }
    
    fn print_detailed_error(&self, error: &PapercraftError) {
        match error {
            PapercraftError::FileNotFound { path } => {
                eprintln!("   📁 Searched in: {}", path.display());
                if let Some(parent) = path.parent() {
                    eprintln!("   📂 Parent directory exists: {}", parent.exists());
                }
            }
            PapercraftError::ConversionFailed { input, output, reason } => {
                eprintln!("   📄 Input file size: {} bytes", 
                    std::fs::metadata(input).map(|m| m.len()).unwrap_or(0));
                eprintln!("   📁 Output directory: {}", 
                    output.parent().map(|p| p.display().to_string()).unwrap_or_default());
                eprintln!("   💬 Detailed reason: {reason}");
            }
            PapercraftError::MemoryLimitExceeded { path: _, size_mb: _ } => {
                eprintln!("   📊 System memory usage may be high");
                eprintln!("   📋 Consider processing in smaller chunks");
            }
            _ => {}
        }
    }
    
    fn print_suggestions(&self, error: &PapercraftError) {
        eprintln!("💡 Suggestions:");
        match error {
            PapercraftError::FileNotFound { .. } => {
                eprintln!("   • Check the file path spelling");
                eprintln!("   • Ensure the file exists and is accessible");
                eprintln!("   • Use absolute paths if relative paths fail");
            }
            PapercraftError::InvalidFileFormat { .. } => {
                eprintln!("   • Only .md (Markdown) files are supported");
                eprintln!("   • Check file extension is correct");
            }
            PapercraftError::PermissionDenied { .. } => {
                eprintln!("   • Run with administrator privileges");
                eprintln!("   • Check file/directory permissions");
                eprintln!("   • Ensure file is not locked by another process");
            }
            PapercraftError::BrowserInitFailed { .. } => {
                eprintln!("   • Ensure Chrome/Chromium is installed");
                eprintln!("   • Close other Chrome instances");
                eprintln!("   • Check system resources");
            }
            PapercraftError::MemoryLimitExceeded { .. } => {
                eprintln!("   • Split large files into smaller sections");
                eprintln!("   • Close other applications to free memory");
                eprintln!("   • Use --optimize-images flag");
            }
            PapercraftError::InsufficientDiskSpace { .. } => {
                eprintln!("   • Free up disk space");
                eprintln!("   • Use a different output directory");
                eprintln!("   • Enable image optimization to reduce file size");
            }
            _ => {
                eprintln!("   • Check the documentation for more details");
                eprintln!("   • Run with --verbose for more information");
            }
        }
    }
}

pub fn validate_input_file(path: &Path) -> Result<(), PapercraftError> {
    if !path.exists() {
        return Err(PapercraftError::FileNotFound { path: path.to_path_buf() });
    }
    
    if !path.is_file() {
        return Err(PapercraftError::InvalidFileFormat { path: path.to_path_buf() });
    }
    
    // Enhanced file extension validation
    let valid_extensions = ["md", "markdown", "mdown", "mkd"];
    let has_valid_extension = path.extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext| valid_extensions.contains(&ext.to_lowercase().as_str()));
    
    if !has_valid_extension {
        return Err(PapercraftError::InvalidFileFormat { path: path.to_path_buf() });
    }
    
    // Enhanced file size validation
    if let Ok(metadata) = std::fs::metadata(path) {
        let size_bytes = metadata.len();
        let size_mb = (size_bytes as f64 / (1024.0 * 1024.0)).ceil() as u64;
        
        // Error for files > 500MB
        if size_mb > 500 {
            return Err(PapercraftError::MemoryLimitExceeded { 
                path: path.to_path_buf(), 
                size_mb 
            });
        }
        
        // Warn for files > 50MB
        if size_mb > 50 {
            eprintln!("⚠️  Warning: Large file detected ({size_mb}MB). Processing may take longer.");
        }
        
        // Warn for empty files
        if size_bytes == 0 {
            eprintln!("⚠️  Warning: Empty file detected. Output may be minimal.");
        }
    }
    
    // Validate file permissions
    if let Err(_) = std::fs::File::open(path) {
        return Err(PapercraftError::PermissionDenied { path: path.to_path_buf() });
    }
    
    // Basic content validation
    validate_file_content(path)?;
    
    Ok(())
}

/// Validates the basic structure and content of the markdown file
fn validate_file_content(path: &Path) -> Result<(), PapercraftError> {
    let content = std::fs::read_to_string(path)
        .map_err(|_| PapercraftError::PermissionDenied { path: path.to_path_buf() })?;
    
    // Check for binary content (likely not a text file)
    if content.chars().any(|c| c == '\0' || (c as u32) < 32 && c != '\n' && c != '\r' && c != '\t') {
        return Err(PapercraftError::InvalidFileFormat { path: path.to_path_buf() });
    }
    
    // Check for extremely long lines (> 10000 characters) which might indicate binary data
    if content.lines().any(|line| line.len() > 10000) {
        eprintln!("⚠️  Warning: Very long lines detected in {}. This might affect rendering.", path.display());
    }
    
    // Check for valid UTF-8 (already handled by read_to_string, but let's be explicit)
    if !content.is_ascii() && content.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
        eprintln!("⚠️  Warning: Non-standard characters detected in {}.", path.display());
    }
    
    Ok(())
}

#[allow(dead_code)]
pub fn check_disk_space(_output_path: &Path, _estimated_size_mb: u64) -> Result<(), PapercraftError> {
    // Simple disk space check - in a real implementation, you'd use platform-specific APIs
    // For now, we'll skip the actual check but keep the structure
    Ok(())
}

#[allow(dead_code)]
pub fn wrap_with_context<T>(
    result: Result<T>,
    input_path: &Path,
    output_path: &Path,
) -> Result<T> {
    result.with_context(|| format!(
        "Failed to convert {} to {}",
        input_path.display(),
        output_path.display()
    ))
}

/// Validates that a path doesn't contain directory traversal attempts
pub fn validate_output_path(path: &Path, base_dir: &Path) -> Result<(), PapercraftError> {
    // Canonicalize paths to resolve any '..' or '.' components
    let canonical_path = path.canonicalize()
        .or_else(|_| {
            // If canonicalize fails (path doesn't exist), try with parent directory
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    parent.canonicalize().map(|p| p.join(path.file_name().unwrap()))
                } else {
                    std::env::current_dir().map(|p| p.join(path))
                }
            } else {
                std::env::current_dir().map(|p| p.join(path))
            }
        })
        .map_err(|_| PapercraftError::PathTraversalAttempt { path: path.to_path_buf() })?;
    
    let canonical_base = base_dir.canonicalize()
        .map_err(|_| PapercraftError::PathTraversalAttempt { path: base_dir.to_path_buf() })?;
    
    // Check if the canonical path starts with the canonical base directory
    if !canonical_path.starts_with(&canonical_base) {
        return Err(PapercraftError::PathTraversalAttempt { path: path.to_path_buf() });
    }
    
    // Additional check for suspicious path components
    for component in path.components() {
        if let std::path::Component::Normal(comp) = component {
            let comp_str = comp.to_string_lossy();
            if comp_str.contains("..") || comp_str.starts_with('.') && comp_str.len() > 1 {
                return Err(PapercraftError::PathTraversalAttempt { path: path.to_path_buf() });
            }
        }
    }
    
    Ok(())
}

/// Sanitizes a file name by removing/replacing dangerous characters
#[allow(dead_code)]
pub fn sanitize_filename(filename: &str) -> String {
    let dangerous_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    let mut sanitized = filename.to_string();
    
    for ch in dangerous_chars {
        sanitized = sanitized.replace(ch, "_");
    }
    
    // Remove leading/trailing dots and spaces
    sanitized = sanitized.trim_matches(|c| c == '.' || c == ' ').to_string();
    
    // Ensure filename is not empty and not reserved names on Windows
    if sanitized.is_empty() || is_reserved_name(&sanitized) {
        sanitized = format!("sanitized_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs());
    }
    
    sanitized
}

#[allow(dead_code)]
fn is_reserved_name(name: &str) -> bool {
    let reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", 
                   "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", 
                   "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
    reserved.contains(&name.to_uppercase().as_str())
}