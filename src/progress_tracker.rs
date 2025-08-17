use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::sync::Arc;
use std::time::Duration;
use parking_lot::Mutex;

pub struct ProgressTracker {
    multi_progress: Arc<MultiProgress>,
    main_bar: Option<Arc<Mutex<ProgressBar>>>,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            multi_progress: Arc::new(MultiProgress::new()),
            main_bar: None,
        }
    }
    
    pub fn start_batch_progress(&mut self, total_files: u64) -> Arc<Mutex<ProgressBar>> {
        let pb = self.multi_progress.add(ProgressBar::new(total_files));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files {elapsed}")
                .unwrap()
                .progress_chars("#>-")
        );
        pb.set_message("Processing files...");
        
        let arc_pb = Arc::new(Mutex::new(pb));
        self.main_bar = Some(arc_pb.clone());
        arc_pb
    }
    
    pub fn create_file_progress(&self, file_name: &str, estimated_size_kb: u64) -> Arc<Mutex<ProgressBar>> {
        let pb = self.multi_progress.add(ProgressBar::new(100));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  {spinner:.green} {msg} [{bar:30.cyan/blue}] {percent}%")
                .unwrap()
                .progress_chars("#>-")
        );
        pb.set_message(format!("📄 {}", file_name));
        
        // For large files, show more detailed progress
        if estimated_size_kb > 1000 {
            pb.enable_steady_tick(Duration::from_millis(120));
        }
        
        Arc::new(Mutex::new(pb))
    }
    
    pub fn finish_file_progress(&self, pb: Arc<Mutex<ProgressBar>>, success: bool) {
        let progress = pb.lock();
        if success {
            progress.finish_with_message("✓ Complete");
        } else {
            progress.finish_with_message("✗ Failed");
        }
    }
    
    pub fn update_batch_progress(&self, increment: u64) {
        if let Some(ref main_bar) = self.main_bar {
            main_bar.lock().inc(increment);
        }
    }
    
    pub fn finish_batch(&self) {
        if let Some(ref main_bar) = self.main_bar {
            let pb = main_bar.lock();
            pb.finish_with_message("🎉 Batch processing complete!");
        }
    }
    
    pub fn set_file_stage(&self, pb: Arc<Mutex<ProgressBar>>, stage: &str, progress: u64) {
        let progress_bar = pb.lock();
        progress_bar.set_message(stage.to_string());
        progress_bar.set_position(progress);
    }
}

pub struct FileProgressStages;

impl FileProgressStages {
    pub const READING: &'static str = "📖 Reading file...";
    #[allow(dead_code)]
    pub const PARSING: &'static str = "🔍 Parsing markdown...";
    #[allow(dead_code)]
    pub const PROCESSING: &'static str = "⚙️  Processing content...";
    #[allow(dead_code)]
    pub const IMAGES: &'static str = "🖼️  Optimizing images...";
    #[allow(dead_code)]
    pub const HTML: &'static str = "🌐 Converting to HTML...";
    #[allow(dead_code)]
    pub const PDF: &'static str = "📄 Generating PDF...";
    pub const FINALIZING: &'static str = "✨ Finalizing...";
}

#[allow(dead_code)]
pub fn estimate_file_processing_time(file_size_kb: u64) -> Duration {
    // Rough estimation: 1KB = 1ms, with minimum of 500ms
    let base_time = std::cmp::max(500, file_size_kb);
    Duration::from_millis(base_time)
}

pub fn estimate_file_size_kb(input_path: &std::path::Path) -> u64 {
    std::fs::metadata(input_path)
        .map(|metadata| {
            let size_kb = (metadata.len() as f64 / 1024.0).ceil() as u64;
            std::cmp::max(size_kb, 1) // Minimum 1KB for any file
        })
        .unwrap_or(10) // Default to 10KB if we can't read the file
}