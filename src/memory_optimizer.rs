use std::path::Path;
use std::io::{BufRead, BufReader};
use std::fs::File;
use anyhow::{Result, Context};

pub struct MemoryOptimizer {
    #[allow(dead_code)]
    max_memory_mb: u64,
    chunk_size_lines: usize,
}

impl MemoryOptimizer {
    pub fn new(max_memory_mb: u64) -> Self {
        Self {
            max_memory_mb,
            chunk_size_lines: 1000, // Process 1000 lines at a time for large files
        }
    }
    
    #[allow(dead_code)]
    pub fn should_use_chunked_processing(&self, file_path: &Path) -> Result<bool> {
        let metadata = std::fs::metadata(file_path)
            .context("Failed to read file metadata")?;
        
        let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        
        // Use chunked processing for files > 10MB or if they exceed our memory limit
        Ok(file_size_mb > 10.0 || file_size_mb > (self.max_memory_mb as f64) / 4.0)
    }
    
    pub fn get_file_info(&self, file_path: &Path) -> Result<FileInfo> {
        let metadata = std::fs::metadata(file_path)
            .context("Failed to read file metadata")?;
        
        let file_size_bytes = metadata.len();
        let file_size_mb = (file_size_bytes as f64 / (1024.0 * 1024.0)).ceil() as u64;
        
        // Count lines for better chunking estimation
        let line_count = self.count_lines(file_path)?;
        
        Ok(FileInfo {
            size_bytes: file_size_bytes,
            size_mb: file_size_mb,
            line_count,
            estimated_chunks: if line_count > self.chunk_size_lines {
                (line_count / self.chunk_size_lines) + 1
            } else {
                1
            },
        })
    }
    
    fn count_lines(&self, file_path: &Path) -> Result<usize> {
        let file = File::open(file_path)
            .context("Failed to open file for line counting")?;
        let reader = BufReader::new(file);
        
        Ok(reader.lines().count())
    }
    
    #[allow(dead_code)]
    pub fn read_file_chunked<F>(&self, file_path: &Path, mut process_chunk: F) -> Result<String>
    where
        F: FnMut(&str) -> Result<String>,
    {
        let file = File::open(file_path)
            .context("Failed to open file for chunked reading")?;
        let reader = BufReader::new(file);
        
        let mut result = String::new();
        let mut current_chunk = String::new();
        let mut line_count = 0;
        
        for line in reader.lines() {
            let line = line.context("Failed to read line")?;
            current_chunk.push_str(&line);
            current_chunk.push('\n');
            line_count += 1;
            
            if line_count >= self.chunk_size_lines {
                let processed = process_chunk(&current_chunk)?;
                result.push_str(&processed);
                
                // Clear the chunk to free memory
                current_chunk.clear();
                line_count = 0;
                
                // Force garbage collection hint
                if result.len() > 1024 * 1024 {
                    // If result is getting large, we might want to write to temporary files
                    // For now, just continue accumulating
                }
            }
        }
        
        // Process remaining chunk
        if !current_chunk.is_empty() {
            let processed = process_chunk(&current_chunk)?;
            result.push_str(&processed);
        }
        
        Ok(result)
    }
    
    #[allow(dead_code)]
    pub fn optimize_html_content(&self, html: &str) -> String {
        // Remove excessive whitespace and optimize HTML for memory usage
        html.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    #[allow(dead_code)]
    pub fn get_memory_usage_mb(&self) -> u64 {
        // Simple memory usage estimation
        // In a real implementation, you'd use platform-specific APIs
        0 // Placeholder
    }
    
    #[allow(dead_code)]
    pub fn recommend_chunk_size(&self, file_size_mb: u64) -> usize {
        match file_size_mb {
            0..=1 => usize::MAX, // No chunking needed
            2..=10 => 2000,
            11..=50 => 1000,
            51..=100 => 500,
            _ => 250,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct FileInfo {
    #[allow(dead_code)]
    pub size_bytes: u64,
    pub size_mb: u64,
    #[allow(dead_code)]
    pub line_count: usize,
    #[allow(dead_code)]
    pub estimated_chunks: usize,
}

impl FileInfo {
    #[allow(dead_code)]
    pub fn is_large_file(&self) -> bool {
        self.size_mb > 10 || self.line_count > 5000
    }
    
    #[allow(dead_code)]
    pub fn estimated_processing_time_seconds(&self) -> u64 {
        // Rough estimation: 1MB = 2 seconds, minimum 1 second
        std::cmp::max(1, self.size_mb * 2)
    }
}

#[allow(dead_code)]
pub struct MemoryMonitor {
    peak_usage_mb: parking_lot::Mutex<u64>,
}

#[allow(dead_code)]
impl MemoryMonitor {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            peak_usage_mb: parking_lot::Mutex::new(0),
        }
    }
    
    #[allow(dead_code)]
    pub fn update_peak_usage(&self, current_mb: u64) {
        let mut peak = self.peak_usage_mb.lock();
        if current_mb > *peak {
            *peak = current_mb;
        }
    }
    
    #[allow(dead_code)]
    pub fn get_peak_usage_mb(&self) -> u64 {
        *self.peak_usage_mb.lock()
    }
    
    #[allow(dead_code)]
    pub fn check_memory_pressure(&self, limit_mb: u64) -> bool {
        self.get_peak_usage_mb() > limit_mb * 80 / 100 // 80% of limit
    }
}