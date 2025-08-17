use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::config::Config;
use crate::markdown_validator::{MarkdownValidator, ValidationResult};
use crate::logger::Logger;

pub struct DryRunResult {
    pub files_to_process: Vec<FileAnalysis>,
    pub total_size_bytes: u64,
    pub estimated_time_seconds: u64,
    pub validation_results: Vec<(PathBuf, ValidationResult)>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileAnalysis {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub size_bytes: u64,
    pub size_mb: f64,
    pub estimated_time_seconds: u64,
    #[allow(dead_code)]
    pub needs_validation: bool,
    #[allow(dead_code)]
    pub output_exists: bool,
    pub will_overwrite: bool,
}

pub struct DryRunProcessor {
    #[allow(dead_code)]
    config: Config,
    validate_markdown: bool,
    check_overwrite: bool,
}

impl DryRunProcessor {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            validate_markdown: true,
            check_overwrite: true,
        }
    }
    
    pub fn with_validation(mut self, validate: bool) -> Self {
        self.validate_markdown = validate;
        self
    }
    
    #[allow(dead_code)]
    pub fn with_overwrite_check(mut self, check: bool) -> Self {
        self.check_overwrite = check;
        self
    }
    
    pub fn analyze_single_file(&self, input_path: &Path, output_path: &Path) -> Result<DryRunResult> {
        Logger::dry_run(&format!("Analyzing single file: {}", input_path.display()));
        
        let mut files_to_process = Vec::new();
        let mut validation_results = Vec::new();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        
        // Validate input file exists
        if !input_path.exists() {
            errors.push(format!("Input file does not exist: {}", input_path.display()));
            return Ok(DryRunResult {
                files_to_process,
                total_size_bytes: 0,
                estimated_time_seconds: 0,
                validation_results,
                warnings,
                errors,
            });
        }
        
        // Analyze the file
        let analysis = self.analyze_file(input_path, output_path)?;
        let total_size_bytes = analysis.size_bytes;
        let estimated_time_seconds = analysis.estimated_time_seconds;
        
        // Validate markdown if enabled
        if self.validate_markdown {
            Logger::validation(&format!("Validating markdown: {}", input_path.display()));
            let validator = MarkdownValidator::new().with_base_path(
                input_path.parent().unwrap_or(Path::new("."))
            );
            
            match validator.validate_file(input_path) {
                Ok(result) => {
                    if !result.is_valid() {
                        warnings.push(format!("Validation issues found in {}", input_path.display()));
                    }
                    validation_results.push((input_path.to_path_buf(), result));
                }
                Err(e) => {
                    errors.push(format!("Failed to validate {}: {}", input_path.display(), e));
                }
            }
        }
        
        // Check for potential issues
        self.check_file_issues(&analysis, &mut warnings, &mut errors);
        
        files_to_process.push(analysis);
        
        Ok(DryRunResult {
            files_to_process,
            total_size_bytes,
            estimated_time_seconds,
            validation_results,
            warnings,
            errors,
        })
    }
    
    pub fn analyze_batch(&self, input_dir: &Path, output_dir: &Path) -> Result<DryRunResult> {
        Logger::dry_run(&format!("Analyzing batch processing: {}", input_dir.display()));
        
        let mut files_to_process = Vec::new();
        let mut validation_results = Vec::new();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut total_size_bytes = 0;
        let mut estimated_time_seconds = 0;
        
        // Validate input directory
        if !input_dir.exists() {
            errors.push(format!("Input directory does not exist: {}", input_dir.display()));
            return Ok(DryRunResult {
                files_to_process,
                total_size_bytes,
                estimated_time_seconds,
                validation_results,
                warnings,
                errors,
            });
        }
        
        // Collect all markdown files
        let mut markdown_files = Vec::new();
        for entry in WalkDir::new(input_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                markdown_files.push(path.to_path_buf());
            }
        }
        
        if markdown_files.is_empty() {
            warnings.push(format!("No markdown files found in directory: {}", input_dir.display()));
            return Ok(DryRunResult {
                files_to_process,
                total_size_bytes,
                estimated_time_seconds,
                validation_results,
                warnings,
                errors,
            });
        }
        
        Logger::dry_run(&format!("Found {} markdown files to process", markdown_files.len()));
        
        // Analyze each file
        for input_path in markdown_files {
            let relative_path = input_path.strip_prefix(input_dir)
                .context("Failed to calculate relative path")?;
            let output_path = output_dir.join(relative_path).with_extension("pdf");
            
            match self.analyze_file(&input_path, &output_path) {
                Ok(analysis) => {
                    total_size_bytes += analysis.size_bytes;
                    estimated_time_seconds += analysis.estimated_time_seconds;
                    
                    // Validate markdown if enabled
                    if self.validate_markdown {
                        let validator = MarkdownValidator::new().with_base_path(
                            input_path.parent().unwrap_or(Path::new("."))
                        );
                        
                        match validator.validate_file(&input_path) {
                            Ok(result) => {
                                if !result.is_valid() {
                                    warnings.push(format!("Validation issues in {}", input_path.display()));
                                }
                                validation_results.push((input_path.clone(), result));
                            }
                            Err(e) => {
                                errors.push(format!("Validation failed for {}: {}", input_path.display(), e));
                            }
                        }
                    }
                    
                    // Check for issues
                    self.check_file_issues(&analysis, &mut warnings, &mut errors);
                    
                    files_to_process.push(analysis);
                }
                Err(e) => {
                    errors.push(format!("Failed to analyze {}: {}", input_path.display(), e));
                }
            }
        }
        
        // Check output directory issues
        if !output_dir.exists() {
            warnings.push(format!("Output directory will be created: {}", output_dir.display()));
        }
        
        Ok(DryRunResult {
            files_to_process,
            total_size_bytes,
            estimated_time_seconds,
            validation_results,
            warnings,
            errors,
        })
    }
    
    fn analyze_file(&self, input_path: &Path, output_path: &Path) -> Result<FileAnalysis> {
        let metadata = std::fs::metadata(input_path)
            .with_context(|| format!("Failed to read metadata for {}", input_path.display()))?;
        
        let size_bytes = metadata.len();
        let size_mb = size_bytes as f64 / (1024.0 * 1024.0);
        
        // Estimate processing time (rough calculation)
        let base_time = 2; // 2 seconds minimum
        let size_factor = (size_mb * 2.0) as u64; // 2 seconds per MB
        let estimated_time_seconds = base_time + size_factor;
        
        let output_exists = output_path.exists();
        let will_overwrite = output_exists;
        
        // Check if validation is needed (always true for now)
        let needs_validation = true;
        
        Ok(FileAnalysis {
            input_path: input_path.to_path_buf(),
            output_path: output_path.to_path_buf(),
            size_bytes,
            size_mb,
            estimated_time_seconds,
            needs_validation,
            output_exists,
            will_overwrite,
        })
    }
    
    fn check_file_issues(&self, analysis: &FileAnalysis, warnings: &mut Vec<String>, _errors: &mut Vec<String>) {
        // Check for large files
        if analysis.size_mb > 10.0 {
            warnings.push(format!(
                "Large file detected: {} ({:.1} MB) - processing may take longer",
                analysis.input_path.display(),
                analysis.size_mb
            ));
        }
        
        // Check for very large files
        if analysis.size_mb > 50.0 {
            warnings.push(format!(
                "Very large file: {} ({:.1} MB) - consider splitting or using memory optimization",
                analysis.input_path.display(),
                analysis.size_mb
            ));
        }
        
        // Check for overwrite
        if self.check_overwrite && analysis.will_overwrite {
            warnings.push(format!(
                "Output file will be overwritten: {}",
                analysis.output_path.display()
            ));
        }
        
        // Check for output directory creation
        if let Some(parent) = analysis.output_path.parent() {
            if !parent.exists() {
                warnings.push(format!(
                    "Output directory will be created: {}",
                    parent.display()
                ));
            }
        }
        
        // Check file permissions (simplified)
        if analysis.input_path.metadata().map_or(false, |m| m.permissions().readonly()) {
            warnings.push(format!(
                "Input file is read-only: {}",
                analysis.input_path.display()
            ));
        }
    }
}

impl DryRunResult {
    pub fn print_summary(&self) {
        Logger::dry_run("📊 Dry Run Summary");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        // Files summary
        println!("📁 Files to process: {}", self.files_to_process.len());
        if !self.files_to_process.is_empty() {
            println!("📊 Total size: {:.2} MB", self.total_size_bytes as f64 / (1024.0 * 1024.0));
            println!("⏱️  Estimated time: {} seconds", self.estimated_time_seconds);
        }
        
        // Validation summary
        if !self.validation_results.is_empty() {
            println!("🔍 Validation results:");
            let mut total_errors = 0;
            let mut total_warnings = 0;
            
            for (_, result) in &self.validation_results {
                total_errors += result.stats.errors;
                total_warnings += result.stats.warnings;
            }
            
            println!("  ❌ Total errors: {}", total_errors);
            println!("  ⚠️  Total warnings: {}", total_warnings);
        }
        
        // Issues summary
        if !self.errors.is_empty() {
            println!("\n❌ Errors found:");
            for error in &self.errors {
                println!("  • {}", error);
            }
        }
        
        if !self.warnings.is_empty() {
            println!("\n⚠️  Warnings:");
            for warning in &self.warnings {
                println!("  • {}", warning);
            }
        }
        
        if self.errors.is_empty() && self.warnings.is_empty() {
            println!("\n✅ No issues detected!");
        }
        
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
    
    pub fn print_detailed_analysis(&self, show_validation_details: bool) {
        if self.files_to_process.is_empty() {
            Logger::warning("No files to analyze");
            return;
        }
        
        Logger::dry_run("📋 Detailed File Analysis");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        for (i, file) in self.files_to_process.iter().enumerate() {
            println!("\n📄 File {} of {}", i + 1, self.files_to_process.len());
            println!("  📂 Input:  {}", file.input_path.display());
            println!("  📄 Output: {}", file.output_path.display());
            println!("  📊 Size:   {:.2} MB ({} bytes)", file.size_mb, file.size_bytes);
            println!("  ⏱️  Time:   ~{} seconds", file.estimated_time_seconds);
            
            if file.will_overwrite {
                println!("  ⚠️  Will overwrite existing output file");
            }
            
            // Show validation details if requested
            if show_validation_details {
                if let Some((_, validation)) = self.validation_results.iter()
                    .find(|(path, _)| path == &file.input_path) 
                {
                    println!("  🔍 Validation:");
                    println!("    📝 Lines: {}", validation.stats.total_lines);
                    println!("    🔗 Links: {}", validation.stats.links);
                    println!("    🖼️  Images: {}", validation.stats.images);
                    println!("    💻 Code blocks: {}", validation.stats.code_blocks);
                    
                    if validation.stats.errors > 0 || validation.stats.warnings > 0 {
                        println!("    ❌ Errors: {}", validation.stats.errors);
                        println!("    ⚠️  Warnings: {}", validation.stats.warnings);
                    } else {
                        println!("    ✅ No validation issues");
                    }
                }
            }
        }
        
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
    
    pub fn has_blocking_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn can_proceed(&self) -> bool {
        !self.has_blocking_errors() && !self.files_to_process.is_empty()
    }
}