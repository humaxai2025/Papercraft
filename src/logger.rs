use std::sync::atomic::{AtomicU8, Ordering};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Quiet = 0,
    Normal = 1,
    Verbose = 2,
    Debug = 3,
}

impl From<u8> for LogLevel {
    fn from(value: u8) -> Self {
        match value {
            0 => LogLevel::Quiet,
            1 => LogLevel::Normal,
            2 => LogLevel::Verbose,
            3 => LogLevel::Debug,
            _ => LogLevel::Normal,
        }
    }
}

static LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Normal as u8);

pub struct Logger;

impl Logger {
    pub fn set_level(level: LogLevel) {
        LOG_LEVEL.store(level as u8, Ordering::Relaxed);
    }
    
    pub fn get_level() -> LogLevel {
        LogLevel::from(LOG_LEVEL.load(Ordering::Relaxed))
    }
    
    #[allow(dead_code)]
    pub fn quiet<T: Display>(message: T) {
        if Self::get_level() != LogLevel::Quiet {
            println!("{message}");
        }
    }
    
    pub fn info<T: Display>(message: T) {
        if Self::get_level() >= LogLevel::Normal {
            println!("{message}");
        }
    }
    
    pub fn verbose<T: Display>(message: T) {
        if Self::get_level() >= LogLevel::Verbose {
            println!("🔍 {}", message);
        }
    }
    
    #[allow(dead_code)]
    pub fn debug<T: Display>(message: T) {
        if Self::get_level() >= LogLevel::Debug {
            println!("🐛 DEBUG: {}", message);
        }
    }
    
    pub fn success<T: Display>(message: T) {
        if Self::get_level() >= LogLevel::Normal {
            println!("✓ {}", message);
        }
    }
    
    pub fn warning<T: Display>(message: T) {
        if Self::get_level() >= LogLevel::Normal {
            println!("⚠️  {}", message);
        }
    }
    
    pub fn error<T: Display>(message: T) {
        // Always show errors unless completely quiet
        if Self::get_level() != LogLevel::Quiet {
            eprintln!("❌ {}", message);
        }
    }
    
    pub fn step<T: Display>(step: u32, total: u32, message: T) {
        if Self::get_level() >= LogLevel::Normal {
            println!("[{}/{}] {}", step, total, message);
        }
    }
    
    pub fn progress<T: Display>(message: T) {
        if Self::get_level() >= LogLevel::Normal {
            println!("🔄 {}", message);
        }
    }
    
    pub fn dry_run<T: Display>(message: T) {
        if Self::get_level() >= LogLevel::Normal {
            println!("🏃 DRY RUN: {}", message);
        }
    }
    
    pub fn validation<T: Display>(message: T) {
        if Self::get_level() >= LogLevel::Verbose {
            println!("🔍 VALIDATION: {}", message);
        }
    }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::Logger::info(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_verbose {
    ($($arg:tt)*) => {
        $crate::logger::Logger::verbose(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::Logger::debug(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_success {
    ($($arg:tt)*) => {
        $crate::logger::Logger::success(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {
        $crate::logger::Logger::warning(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logger::Logger::error(format!($($arg)*))
    };
}