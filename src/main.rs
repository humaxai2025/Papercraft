use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::WalkDir;
use notify::{Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

mod html_converter;
mod config;
mod themes;
mod advanced_processing;
mod image_optimization;
mod advanced_styles;
mod error_handler;
mod progress_tracker;
mod memory_optimizer;
mod resume_handler;
mod logger;
mod markdown_validator;
mod config_wizard;
mod dry_run;
mod chrome_manager;

use html_converter::{ConversionOptions, HtmlToPdfConverter};
use config::Config;
use error_handler::{ErrorReporter, PapercraftError};
use progress_tracker::{ProgressTracker, FileProgressStages};
use memory_optimizer::MemoryOptimizer;
use resume_handler::{ResumeHandler, JobStatus};
use rayon::prelude::*;
use std::sync::Arc;
use parking_lot::Mutex;
use logger::{Logger, LogLevel};
use markdown_validator::MarkdownValidator;
use config_wizard::ConfigWizard;
use dry_run::DryRunProcessor;

#[derive(Parser, Debug)]
#[command(
    name = "papercraft",
    about = "🎨 PaperCraft - A professional Markdown to PDF converter with beautiful themes and advanced configuration.",
    version = "1.0.0"
)]
struct Args {
    /// Input Markdown file or directory for batch processing
    #[arg(short, long, value_name = "FILE/DIR")]
    input: Option<PathBuf>,

    /// Output PDF file or directory for batch processing
    #[arg(short, long, value_name = "FILE/DIR")]
    output: Option<PathBuf>,

    /// Enable batch processing mode
    #[arg(long)]
    batch: bool,

    /// Watch directory for changes and auto-regenerate
    #[arg(long)]
    watch: bool,

    /// Path to configuration file (TOML, YAML, JSON)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Built-in theme to use (default, dark, minimal, academic, modern)
    #[arg(long)]
    theme: Option<String>,

    /// Path to a custom CSS theme file
    #[arg(long, value_name = "FILE")]
    theme_file: Option<PathBuf>,

    /// Paper size (A4, Letter, Legal, A3, A5)
    #[arg(long)]
    paper_size: Option<String>,

    /// Page orientation (portrait, landscape)
    #[arg(long)]
    orientation: Option<String>,

    /// Page margins (e.g., "1in" or "20mm")
    #[arg(long)]
    margins: Option<String>,

    /// Font family for the document
    #[arg(long)]
    font_family: Option<String>,

    /// Font size for the document
    #[arg(long)]
    font_size: Option<String>,

    /// Enable page numbers
    #[arg(long)]
    page_numbers: bool,

    /// Page number format (e.g., "Page {page} of {total}")
    #[arg(long)]
    page_number_format: Option<String>,

    /// HTML template for the page header
    #[arg(long)]
    header_template: Option<String>,

    /// HTML template for the page footer
    #[arg(long)]
    footer_template: Option<String>,

    /// Generate a sample configuration file
    #[arg(long)]
    generate_config: Option<PathBuf>,

    /// Show version information
    #[arg(long)]
    show_version: bool,

    /// List available built-in themes
    #[arg(long)]
    list_themes: bool,

    /// Enable table of contents
    #[arg(long)]
    toc: bool,

    /// Disable table of contents
    #[arg(long)]
    no_toc: bool,

    /// Enable code line numbers
    #[arg(long)]
    line_numbers: bool,

    /// Enable footnotes
    #[arg(long)]
    footnotes: bool,

    /// Enable bibliography
    #[arg(long)]
    bibliography: bool,

    /// Enable image optimization
    #[arg(long)]
    optimize_images: bool,

    /// Maximum image width in pixels
    #[arg(long)]
    max_image_width: Option<u32>,

    /// Maximum image height in pixels
    #[arg(long)]
    max_image_height: Option<u32>,

    /// Enable verbose error reporting
    #[arg(long)]
    verbose: bool,

    /// Enable concurrent processing for batch operations
    #[arg(long)]
    concurrent: bool,

    /// Number of concurrent jobs (default: number of CPU cores)
    #[arg(long)]
    jobs: Option<usize>,

    /// Maximum memory usage in MB (default: 1024)
    #[arg(long)]
    max_memory: Option<u64>,

    /// Resume incomplete batch job by ID
    #[arg(long)]
    resume: Option<String>,

    /// List incomplete jobs that can be resumed
    #[arg(long)]
    list_jobs: bool,

    /// Cancel a running batch job by ID
    #[arg(long)]
    cancel_job: Option<String>,

    /// Enable quiet mode (minimal output)
    #[arg(short, long)]
    quiet: bool,

    /// Enable debug logging
    #[arg(long)]
    debug: bool,

    /// Dry run mode - preview changes without converting
    #[arg(long)]
    dry_run: bool,

    /// Validate markdown files before conversion
    #[arg(long)]
    validate: bool,

    /// Skip markdown validation
    #[arg(long)]
    no_validate: bool,

    /// Run configuration wizard for first-time setup
    #[arg(long)]
    setup_wizard: bool,

    /// Show detailed validation results in dry run
    #[arg(long)]
    show_validation_details: bool,
    
    /// Check Chrome Headless Shell status and download if needed (optional - Chrome downloads automatically when needed)
    #[arg(long)]
    check_chrome: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging level
    let log_level = if args.quiet {
        LogLevel::Quiet
    } else if args.debug {
        LogLevel::Debug
    } else if args.verbose {
        LogLevel::Verbose
    } else {
        LogLevel::Normal
    };
    Logger::set_level(log_level);

    // Initialize error reporter
    let error_reporter = ErrorReporter::new(args.verbose);

    // Initialize resume handler
    let resume_handler = ResumeHandler::new(".papercraft_state")
        .context("Failed to initialize resume handler")?;

    // Handle special commands first
    if args.show_version {
        Logger::info(&format!("🎨 PaperCraft v{}", env!("CARGO_PKG_VERSION")));
        Logger::info("A professional Markdown to PDF converter with advanced features");
        Logger::info("https://github.com/papercraft/papercraft");
        return Ok(());
    }

    // Configuration wizard
    if args.setup_wizard {
        let wizard = ConfigWizard::new();
        let config = wizard.run_interactive_setup()?;
        wizard.save_config_with_wizard(&config, Some(PathBuf::from("papercraft.toml")))?;
        return Ok(());
    }
    
    if args.check_chrome {
        let converter = HtmlToPdfConverter::new()
            .context("Failed to initialize converter")?;
        converter.check_chrome_status()?;
        
        // Try to ensure Chrome is available
        use crate::chrome_manager::ChromeManager;
        let chrome_manager = ChromeManager::new()?;
        let chrome_path = chrome_manager.ensure_chrome()?;
        let version = chrome_manager.get_chrome_version()?;
        
        println!("✅ Chrome Headless Shell ready: {}", chrome_path.display());
        println!("📋 Version: {}", version);
        return Ok(());
    }

    if args.list_themes {
        let theme_manager = themes::ThemeManager::new();
        println!("Available built-in themes:");
        for theme in theme_manager.list_built_in_themes() {
            println!("  - {}", theme);
        }
        return Ok(());
    }

    if let Some(config_path) = &args.generate_config {
        let config = Config::default();
        config.save_to_file(config_path)
            .with_context(|| format!("Failed to generate config file: {}", config_path.display()))?;
        println!("✓ Generated sample configuration file: {}", config_path.display());
        return Ok(());
    }

    // Handle resume functionality
    if args.list_jobs {
        let incomplete_jobs = resume_handler.list_incomplete_jobs()?;
        if incomplete_jobs.is_empty() {
            println!("📝 No incomplete jobs found");
        } else {
            println!("📝 Incomplete jobs:");
            for job in incomplete_jobs {
                println!("  🔄 {} - {} files ({} completed, {} failed)",
                    job.id, job.total_files, job.completed_files, job.failed_files);
            }
        }
        return Ok(());
    }

    if let Some(job_id) = &args.cancel_job {
        resume_handler.cancel_batch_job(job_id)?;
        println!("❌ Cancelled job: {}", job_id);
        return Ok(());
    }

    if let Some(job_id) = &args.resume {
        if let Some(batch_job) = resume_handler.resume_batch_job(job_id)? {
            println!("🔄 Resuming job: {} ({} files)", batch_job.id, batch_job.total_files);
            // Continue with normal processing using the resumed job
        } else {
            eprintln!("❌ Job not found: {}", job_id);
            return Ok(());
        }
    }

    // Validate required arguments for conversion
    let input = args.input.as_ref().ok_or_else(|| anyhow::anyhow!("Input file or directory is required for conversion"))?;
    let output = args.output.as_ref().ok_or_else(|| anyhow::anyhow!("Output file or directory is required for conversion"))?;

    if fs::metadata(input).is_err() {
        anyhow::bail!("Input path does not exist: {}", input.display());
    }

    // Load configuration
    let mut config = if let Some(config_path) = &args.config {
        Config::load_from_file(config_path)
            .with_context(|| format!("Failed to load config file: {}", config_path.display()))?
    } else {
        Config::load_or_default()?
    };

    // Override config with command line arguments
    apply_cli_overrides(&mut config, &args);

    let converter = HtmlToPdfConverter::new()
        .context("Failed to initialize converter")?;

    let options = ConversionOptions { config };

    // Determine if validation should be performed
    let should_validate = if args.no_validate {
        false
    } else if args.validate {
        true
    } else {
        // Default to validation in dry run mode
        args.dry_run
    };

    if args.watch {
        watch_directory(input, output, &converter, options)?;
    } else if args.dry_run {
        run_dry_run_analysis(input, output, &options.config, &args, should_validate)?;
    } else if args.batch || input.is_dir() {
        if should_validate {
            Logger::verbose("Pre-conversion validation enabled");
            validate_before_batch_processing(input)?;
        }
        batch_process_directory(input, output, &converter, options, &args, &error_reporter, &resume_handler)?;
    } else {
        if should_validate {
            Logger::verbose("Pre-conversion validation enabled");
            validate_single_file(input)?;
        }
        single_file_conversion(input, output, &converter, options, &error_reporter)?;
    }

    Ok(())
}

fn run_dry_run_analysis(
    input: &Path,
    output: &Path,
    config: &Config,
    args: &Args,
    validate: bool,
) -> Result<()> {
    Logger::progress("Starting dry run analysis...");
    
    let dry_run_processor = DryRunProcessor::new(config.clone())
        .with_validation(validate);
    
    let result = if input.is_dir() {
        dry_run_processor.analyze_batch(input, output)?
    } else {
        dry_run_processor.analyze_single_file(input, output)?
    };
    
    // Print results
    result.print_summary();
    
    if args.show_validation_details || Logger::get_level() >= LogLevel::Verbose {
        result.print_detailed_analysis(validate);
    }
    
    // Show validation issues if requested
    if validate && !result.validation_results.is_empty() {
        Logger::info("\n🔍 Validation Details:");
        for (file_path, validation_result) in &result.validation_results {
            if !validation_result.issues.is_empty() {
                Logger::info(format!("\n📄 {}", file_path.display()));
                validation_result.print_issues(Logger::get_level() >= LogLevel::Verbose);
            }
        }
    }
    
    if result.has_blocking_errors() {
        Logger::error("❌ Dry run found blocking errors. Please fix them before proceeding.");
        std::process::exit(1);
    } else if result.can_proceed() {
        Logger::success("✅ Dry run completed successfully. Ready to proceed with conversion.");
    } else {
        Logger::warning("⚠️  No files to process.");
    }
    
    Ok(())
}

fn validate_single_file(input: &PathBuf) -> Result<()> {
    Logger::validation(format!("Validating {}", input.display()));
    
    let validator = MarkdownValidator::new().with_base_path(
        input.parent().unwrap_or(std::path::Path::new("."))
    );
    
    match validator.validate_file(input) {
        Ok(result) => {
            if Logger::get_level() >= LogLevel::Verbose {
                result.print_summary();
            }
            
            if !result.is_valid() {
                Logger::warning(format!("Validation issues found in {}", input.display()));
                if Logger::get_level() >= LogLevel::Verbose {
                    result.print_issues(true);
                }
            } else {
                Logger::verbose("✅ Validation passed");
            }
        }
        Err(e) => {
            Logger::warning(format!("Validation failed for {}: {}", input.display(), e));
        }
    }
    
    Ok(())
}

fn validate_before_batch_processing(input_dir: &PathBuf) -> Result<()> {
    Logger::validation(format!("Validating markdown files in {}", input_dir.display()));
    
    let mut total_files = 0;
    let mut files_with_issues = 0;
    
    for entry in WalkDir::new(input_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            total_files += 1;
            
            let validator = MarkdownValidator::new().with_base_path(
                path.parent().unwrap_or(std::path::Path::new("."))
            );
            
            match validator.validate_file(path) {
                Ok(result) => {
                    if !result.is_valid() {
                        files_with_issues += 1;
                        if Logger::get_level() >= LogLevel::Verbose {
                            Logger::warning(format!("Issues in {}", path.display()));
                            result.print_issues(false);
                        }
                    }
                }
                Err(e) => {
                    files_with_issues += 1;
                    Logger::warning(format!("Validation failed for {}: {}", path.display(), e));
                }
            }
        }
    }
    
    Logger::verbose(format!("Validation complete: {total_files} files processed, {files_with_issues} with issues"));
    
    if files_with_issues > 0 {
        Logger::warning(format!("⚠️  {files_with_issues} out of {total_files} files have validation issues"));
    } else {
        Logger::success(format!("✅ All {total_files} files passed validation"));
    }
    
    Ok(())
}

fn apply_cli_overrides(config: &mut Config, args: &Args) {
    // Theme overrides
    if let Some(theme) = &args.theme {
        config.theme.built_in = Some(theme.clone());
        config.theme.css_file = None; // CLI theme takes precedence
    }
    if let Some(theme_file) = &args.theme_file {
        config.theme.css_file = Some(theme_file.clone());
        config.theme.built_in = None; // External file takes precedence
    }

    // Page size override
    if let Some(paper_size) = &args.paper_size {
        config.page.size.preset = Some(paper_size.clone());
        config.page.size.width = None;
        config.page.size.height = None;
    }

    // Orientation override
    if let Some(orientation) = &args.orientation {
        config.page.orientation = match orientation.to_lowercase().as_str() {
            "landscape" => config::Orientation::Landscape,
            _ => config::Orientation::Portrait,
        };
    }

    // Margins override (simplified - applies to all margins)
    if let Some(margins) = &args.margins {
        config.page.margins.top = margins.clone();
        config.page.margins.right = margins.clone();
        config.page.margins.bottom = margins.clone();
        config.page.margins.left = margins.clone();
    }

    // Font overrides
    if let Some(font_family) = &args.font_family {
        config.fonts.family = Some(font_family.clone());
    }
    if let Some(font_size) = &args.font_size {
        config.fonts.size = Some(font_size.clone());
    }

    // Page numbers override
    if args.page_numbers {
        if config.page.page_numbers.is_none() {
            config.page.page_numbers = Some(config::PageNumberConfig {
                enabled: true,
                format: "Page {page} of {total}".to_string(),
                position: config::PageNumberPosition::Footer,
                start_from: Some(1),
            });
        } else if let Some(ref mut page_numbers) = config.page.page_numbers {
            page_numbers.enabled = true;
        }
    }

    // Page number format override
    if let Some(format) = &args.page_number_format {
        if let Some(ref mut page_numbers) = config.page.page_numbers {
            page_numbers.format = format.clone();
        }
    }

    // Header template override
    if let Some(header_template) = &args.header_template {
        config.page.header = Some(config::HeaderFooterConfig {
            enabled: true,
            template: header_template.clone(),
            height: Some("1cm".to_string()),
            font_size: Some("10px".to_string()),
        });
    }

    // Footer template override
    if let Some(footer_template) = &args.footer_template {
        config.page.footer = Some(config::HeaderFooterConfig {
            enabled: true,
            template: footer_template.clone(),
            height: Some("1cm".to_string()),
            font_size: Some("10px".to_string()),
        });
    }

    // Advanced feature overrides
    if args.toc {
        config.toc.enabled = true;
    }
    if args.no_toc {
        config.toc.enabled = false;
    }
    if args.line_numbers {
        config.code.line_numbers = true;
    }
    if args.footnotes {
        config.references.footnotes.enabled = true;
    }
    if args.bibliography {
        config.references.bibliography.enabled = true;
    }
    if args.optimize_images {
        config.images.optimization = true;
    }
    if let Some(max_width) = args.max_image_width {
        config.images.max_width = Some(max_width);
    }
    if let Some(max_height) = args.max_image_height {
        config.images.max_height = Some(max_height);
    }
}

fn single_file_conversion(
    input: &Path,
    output: &Path,
    converter: &HtmlToPdfConverter,
    options: ConversionOptions,
    error_reporter: &ErrorReporter,
) -> Result<()> {
    // Validate input file
    if let Err(e) = error_handler::validate_input_file(input) {
        error_reporter.report_error(&e);
        return Err(anyhow::anyhow!("Input validation failed"));
    }

    println!("📄 Converting: {} → {}", 
        input.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "unknown".into()), 
        output.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "unknown".into()));
    
    // Initialize progress for single file
    #[allow(unused_mut)]
    let mut progress_tracker = ProgressTracker::new();
    let file_size_kb = progress_tracker::estimate_file_size_kb(input);
    let file_progress = progress_tracker.create_file_progress(
        &input.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "unknown".into()),
        file_size_kb
    );

    progress_tracker.set_file_stage(file_progress.clone(), FileProgressStages::READING, 10);
    
    let result = converter.convert_file(input, output, options);
    
    match result {
        Ok(_) => {
            progress_tracker.set_file_stage(file_progress.clone(), FileProgressStages::FINALIZING, 100);
            progress_tracker.finish_file_progress(file_progress, true);
            println!(
                "✓ Successfully converted {} to {}",
                input.display(),
                output.display()
            );
            Ok(())
        }
        Err(e) => {
            progress_tracker.finish_file_progress(file_progress, false);
            let papercraft_error = PapercraftError::ConversionFailed {
                input: input.to_path_buf(),
                output: output.to_path_buf(),
                reason: e.to_string(),
            };
            error_reporter.report_error(&papercraft_error);
            Err(e)
        }
    }
}

fn batch_process_directory(
    input_dir: &PathBuf,
    output_dir: &PathBuf,
    converter: &HtmlToPdfConverter,
    options: ConversionOptions,
    args: &Args,
    error_reporter: &ErrorReporter,
    resume_handler: &ResumeHandler,
) -> Result<()> {
    // Ensure output directory exists
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;
    }

    // Collect all markdown files
    let mut file_paths = Vec::new();
    for entry in WalkDir::new(input_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            file_paths.push(path.to_path_buf());
        }
    }

    if file_paths.is_empty() {
        println!("📁 No markdown files found in directory: {}", input_dir.display());
        return Ok(());
    }

    // Initialize memory optimizer
    let max_memory = args.max_memory.unwrap_or(1024);
    let memory_optimizer = MemoryOptimizer::new(max_memory);

    // Create batch job for resume capability
    let batch_job = resume_handler.create_batch_job(file_paths.clone(), output_dir)?;
    println!("🔄 Starting batch processing: {} ({} files)", batch_job.id, file_paths.len());

    // Initialize progress tracker and start batch progress
    let mut progress_tracker = ProgressTracker::new();
    let _batch_progress = progress_tracker.start_batch_progress(file_paths.len() as u64);

    let processed_count = Arc::new(Mutex::new(0u32));
    let failed_count = Arc::new(Mutex::new(0u32));

    if args.concurrent {
        // Concurrent processing
        let num_threads = args.jobs.unwrap_or_else(num_cpus::get);
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .context("Failed to initialize thread pool")?;

        println!("⚙️  Using {num_threads} concurrent threads");

        // Collect results from parallel processing instead of using try_for_each
        let ctx = ProcessingContext {
            converter,
            options: &options,
            memory_optimizer: &memory_optimizer,
            progress_tracker: &progress_tracker,
            batch_id: &batch_job.id,
            resume_handler,
            error_reporter,
            processed_count: &processed_count,
            failed_count: &failed_count,
        };
        let results: Vec<Result<()>> = file_paths.par_iter().map(|input_path| {
            process_single_file_with_progress(
                input_path,
                input_dir,
                output_dir,
                &ctx,
            )
        }).collect();
        
        // Process results and handle errors gracefully
        for (i, result) in results.into_iter().enumerate() {
            if let Err(e) = result {
                error_reporter.report_error(&PapercraftError::ConversionFailed {
                    input: file_paths[i].clone(),
                    output: output_dir.join(
                        file_paths[i].strip_prefix(input_dir)
                            .unwrap_or(&file_paths[i])
                    ).with_extension("pdf"),
                    reason: e.to_string(),
                });
            }
        }
    } else {
        // Sequential processing
        let ctx = ProcessingContext {
            converter,
            options: &options,
            memory_optimizer: &memory_optimizer,
            progress_tracker: &progress_tracker,
            batch_id: &batch_job.id,
            resume_handler,
            error_reporter,
            processed_count: &processed_count,
            failed_count: &failed_count,
        };
        for input_path in &file_paths {
            if let Err(e) = process_single_file_with_progress(
                input_path,
                input_dir,
                output_dir,
                &ctx,
            ) {
                error_reporter.report_error(&PapercraftError::ConversionFailed {
                    input: input_path.clone(),
                    output: output_dir.join(
                        input_path.strip_prefix(input_dir)
                            .unwrap_or(input_path)
                    ).with_extension("pdf"),
                    reason: e.to_string(),
                });
            }
        }
    }

    progress_tracker.finish_batch();

    let final_processed = *processed_count.lock();
    let final_failed = *failed_count.lock();

    println!("🎉 Batch processing complete!");
    println!("  ✓ Successfully processed: {final_processed} files");
    if final_failed > 0 {
        println!("  ✗ Failed: {final_failed} files");
    }

    Ok(())
}

struct ProcessingContext<'a> {
    converter: &'a HtmlToPdfConverter,
    options: &'a ConversionOptions,
    memory_optimizer: &'a MemoryOptimizer,
    progress_tracker: &'a ProgressTracker,
    batch_id: &'a str,
    resume_handler: &'a ResumeHandler,
    error_reporter: &'a ErrorReporter,
    processed_count: &'a Arc<Mutex<u32>>,
    failed_count: &'a Arc<Mutex<u32>>,
}

fn process_single_file_with_progress(
    input_path: &Path,
    input_dir: &Path,
    output_dir: &Path,
    ctx: &ProcessingContext,
) -> Result<()> {
    let relative_path = input_path.strip_prefix(input_dir)
        .context("Failed to calculate relative path")?;
    
    let output_file = output_dir.join(relative_path).with_extension("pdf");
    
    // Validate output path to prevent directory traversal
    if let Err(e) = error_handler::validate_output_path(&output_file, output_dir) {
        ctx.error_reporter.report_error(&e);
        return Err(anyhow::anyhow!("Path validation failed: {}", e));
    }
    
    // Create parent directory if needed
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Get file info for progress estimation
    let file_info = ctx.memory_optimizer.get_file_info(input_path)?;
    let file_progress = ctx.progress_tracker.create_file_progress(
        &input_path.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "unknown".into()),
        file_info.size_mb * 1024
    );

    let job_id = format!("{}_{}", ctx.batch_id, input_path.to_string_lossy().replace(['\\', '/'], "_"));

    // Update job status to processing
    ctx.resume_handler.update_job_status(ctx.batch_id, &job_id, JobStatus::Processing, 0.0)?;

    // Display file being processed
    println!("📄 Processing: {} → {}", 
        input_path.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "unknown".into()),
        output_file.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "unknown".into()));

    ctx.progress_tracker.set_file_stage(file_progress.clone(), FileProgressStages::READING, 10);

    match ctx.converter.convert_file(input_path, &output_file, ctx.options.clone()) {
        Ok(_) => {
            ctx.progress_tracker.set_file_stage(file_progress.clone(), FileProgressStages::FINALIZING, 90);
            ctx.resume_handler.update_job_status(ctx.batch_id, &job_id, JobStatus::Completed, 100.0)?;
            ctx.progress_tracker.finish_file_progress(file_progress, true);
            ctx.progress_tracker.update_batch_progress(1);
            
            // Display success message
            println!("  ✓ Completed: {}", 
                output_file.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "unknown".into()));
            
            let mut count = ctx.processed_count.lock();
            *count += 1;
        }
        Err(e) => {
            ctx.resume_handler.update_job_status(ctx.batch_id, &job_id, JobStatus::Failed, 0.0)?;
            ctx.progress_tracker.finish_file_progress(file_progress, false);
            
            // Display error message
            println!("  ✗ Failed: {} ({})", 
                input_path.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "unknown".into()),
                e.to_string().lines().next().unwrap_or("Unknown error"));
            
            let mut count = ctx.failed_count.lock();
            *count += 1;
            
            // Don't return error to allow batch processing to continue
            // Error is already logged and counted
        }
    }

    Ok(())
}

fn watch_directory(
    input_dir: &PathBuf,
    output_dir: &PathBuf,
    converter: &HtmlToPdfConverter,
    options: ConversionOptions,
) -> Result<()> {
    use std::sync::mpsc::channel;
    use std::thread;

    println!("👀 Watching directory for changes: {}", input_dir.display());
    println!("📁 Output directory: {}", output_dir.display());
    println!("Press Ctrl+C to stop watching...");

    // Ensure output directory exists
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;
    }

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<Event>| {
            if let Ok(event) = result {
                let _ = tx.send(event);
            }
        },
        NotifyConfig::default(),
    )?;

    watcher.watch(input_dir, RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(event) => {
                if let EventKind::Modify(_) | EventKind::Create(_) = event.kind {
                    for path in &event.paths {
                        if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                            // Debounce: wait a bit to ensure file is fully written
                            thread::sleep(Duration::from_millis(100));
                            
                            if let Ok(relative_path) = path.strip_prefix(input_dir) {
                                let output_file = output_dir.join(relative_path).with_extension("pdf");
                                
                                // Create parent directory if needed
                                if let Some(parent) = output_file.parent() {
                                    let _ = fs::create_dir_all(parent);
                                }

                                println!("🔄 File changed: {} -> {}", path.display(), output_file.display());

                                match converter.convert_file(path, &output_file, options.clone()) {
                                    Ok(_) => println!("  ✓ Regenerated successfully"),
                                    Err(e) => println!("  ✗ Failed to regenerate: {e}"),
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Watch error: {e}");
                break;
            }
        }
    }

    Ok(())
}