use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConversionJob {
    pub id: String,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub progress: f64, // 0.0 to 100.0
    pub error_message: Option<String>,
    pub config_hash: String,
    pub estimated_duration_seconds: u64,
    pub actual_duration_seconds: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum JobStatus {
    Pending,
    Reading,
    Parsing,
    Processing,
    Converting,
    Writing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchJob {
    pub id: String,
    pub jobs: Vec<ConversionJob>,
    pub total_files: usize,
    pub completed_files: usize,
    pub failed_files: usize,
    pub status: BatchStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum BatchStatus {
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

pub struct ResumeHandler {
    state_dir: PathBuf,
}

impl ResumeHandler {
    pub fn new<P: AsRef<Path>>(state_dir: P) -> Result<Self> {
        let state_dir = state_dir.as_ref().to_path_buf();
        
        if !state_dir.exists() {
            std::fs::create_dir_all(&state_dir)
                .context("Failed to create state directory")?;
        }
        
        Ok(Self { state_dir })
    }
    
    pub fn create_batch_job(&self, input_paths: Vec<PathBuf>, output_dir: &Path) -> Result<BatchJob> {
        let job_id = self.generate_job_id();
        let now = Utc::now();
        
        let jobs: Vec<_> = input_paths.into_iter().enumerate().map(|(i, input_path)| {
            let output_path = if input_path.is_file() {
                output_dir.join(input_path.file_stem().unwrap()).with_extension("pdf")
            } else {
                output_dir.join(format!("output_{}.pdf", i))
            };
            
            ConversionJob {
                id: format!("{}_{}", job_id, i),
                input_path,
                output_path,
                status: JobStatus::Pending,
                created_at: now,
                updated_at: now,
                progress: 0.0,
                error_message: None,
                config_hash: self.calculate_config_hash(),
                estimated_duration_seconds: 10, // Default estimate
                actual_duration_seconds: None,
            }
        }).collect();
        
        let total_files = jobs.len();
        
        let batch_job = BatchJob {
            id: job_id,
            jobs,
            total_files,
            completed_files: 0,
            failed_files: 0,
            status: BatchStatus::Running,
            created_at: now,
            updated_at: now,
        };
        
        self.save_batch_job(&batch_job)?;
        Ok(batch_job)
    }
    
    pub fn save_batch_job(&self, batch_job: &BatchJob) -> Result<()> {
        let file_path = self.state_dir.join(format!("{}.json", batch_job.id));
        let json = serde_json::to_string_pretty(batch_job)
            .context("Failed to serialize batch job")?;
        
        std::fs::write(&file_path, json)
            .context("Failed to write batch job state")?;
        
        Ok(())
    }
    
    pub fn load_batch_job(&self, job_id: &str) -> Result<Option<BatchJob>> {
        let file_path = self.state_dir.join(format!("{}.json", job_id));
        
        if !file_path.exists() {
            return Ok(None);
        }
        
        let json = std::fs::read_to_string(&file_path)
            .context("Failed to read batch job state")?;
        
        let batch_job = serde_json::from_str(&json)
            .context("Failed to deserialize batch job")?;
        
        Ok(Some(batch_job))
    }
    
    pub fn update_job_status(&self, batch_id: &str, job_id: &str, status: JobStatus, progress: f64) -> Result<()> {
        if let Some(mut batch_job) = self.load_batch_job(batch_id)? {
            if let Some(job) = batch_job.jobs.iter_mut().find(|j| j.id == job_id) {
                job.status = status.clone();
                job.progress = progress;
                job.updated_at = Utc::now();
                
                // Update batch counters
                match status {
                    JobStatus::Completed => {
                        if job.progress >= 100.0 {
                            batch_job.completed_files += 1;
                        }
                    }
                    JobStatus::Failed => {
                        batch_job.failed_files += 1;
                    }
                    _ => {}
                }
                
                // Update batch status
                if batch_job.completed_files + batch_job.failed_files >= batch_job.total_files {
                    batch_job.status = if batch_job.failed_files == 0 {
                        BatchStatus::Completed
                    } else {
                        BatchStatus::Failed
                    };
                }
                
                batch_job.updated_at = Utc::now();
                self.save_batch_job(&batch_job)?;
            }
        }
        Ok(())
    }
    
    pub fn list_incomplete_jobs(&self) -> Result<Vec<BatchJob>> {
        let mut incomplete_jobs = Vec::new();
        
        for entry in std::fs::read_dir(&self.state_dir)? {
            let entry = entry?;
            if let Some(ext) = entry.path().extension() {
                if ext == "json" {
                    if let Some(stem) = entry.path().file_stem() {
                        let job_id = stem.to_string_lossy();
                        if let Some(batch_job) = self.load_batch_job(&job_id)? {
                            if batch_job.status != BatchStatus::Completed {
                                incomplete_jobs.push(batch_job);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(incomplete_jobs)
    }
    
    pub fn resume_batch_job(&self, batch_id: &str) -> Result<Option<BatchJob>> {
        if let Some(mut batch_job) = self.load_batch_job(batch_id)? {
            // Reset failed jobs to pending for retry
            for job in &mut batch_job.jobs {
                if job.status == JobStatus::Failed {
                    job.status = JobStatus::Pending;
                    job.progress = 0.0;
                    job.error_message = None;
                    job.updated_at = Utc::now();
                }
            }
            
            batch_job.status = BatchStatus::Running;
            batch_job.updated_at = Utc::now();
            
            self.save_batch_job(&batch_job)?;
            Ok(Some(batch_job))
        } else {
            Ok(None)
        }
    }
    
    pub fn cancel_batch_job(&self, batch_id: &str) -> Result<()> {
        if let Some(mut batch_job) = self.load_batch_job(batch_id)? {
            batch_job.status = BatchStatus::Cancelled;
            batch_job.updated_at = Utc::now();
            
            for job in &mut batch_job.jobs {
                if matches!(job.status, JobStatus::Pending | JobStatus::Reading | JobStatus::Parsing | JobStatus::Processing | JobStatus::Converting) {
                    job.status = JobStatus::Cancelled;
                    job.updated_at = Utc::now();
                }
            }
            
            self.save_batch_job(&batch_job)?;
        }
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn cleanup_completed_jobs(&self, older_than_days: u32) -> Result<usize> {
        let cutoff_date = Utc::now() - chrono::Duration::days(older_than_days as i64);
        let mut cleaned_count = 0;
        
        for entry in std::fs::read_dir(&self.state_dir)? {
            let entry = entry?;
            if let Some(ext) = entry.path().extension() {
                if ext == "json" {
                    if let Some(stem) = entry.path().file_stem() {
                        let job_id = stem.to_string_lossy();
                        if let Some(batch_job) = self.load_batch_job(&job_id)? {
                            if batch_job.status == BatchStatus::Completed && batch_job.updated_at < cutoff_date {
                                std::fs::remove_file(entry.path())?;
                                cleaned_count += 1;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(cleaned_count)
    }
    
    fn generate_job_id(&self) -> String {
        format!("job_{}", Utc::now().timestamp())
    }
    
    fn calculate_config_hash(&self) -> String {
        // In a real implementation, this would hash the current configuration
        // For now, return a placeholder
        "config_hash_placeholder".to_string()
    }
}

impl ConversionJob {
    #[allow(dead_code)]
    pub fn is_resumable(&self) -> bool {
        matches!(self.status, JobStatus::Failed | JobStatus::Pending)
    }
    
    #[allow(dead_code)]
    pub fn get_progress_percentage(&self) -> u8 {
        self.progress.clamp(0.0, 100.0) as u8
    }
    
    #[allow(dead_code)]
    pub fn estimate_remaining_time(&self) -> Option<u64> {
        if self.progress > 0.0 {
            let elapsed = self.actual_duration_seconds.unwrap_or(0);
            let _remaining_progress = 100.0 - self.progress;
            let estimated_total = (elapsed as f64 / self.progress) * 100.0;
            Some((estimated_total - elapsed as f64) as u64)
        } else {
            Some(self.estimated_duration_seconds)
        }
    }
}