use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::ZipArchive;

const CHROME_FOR_TESTING_VERSION: &str = "131.0.6778.108"; // Latest stable with headless shell support

#[cfg(target_os = "windows")]
const CHROME_PLATFORM: &str = "win64";
#[cfg(target_os = "linux")]
const CHROME_PLATFORM: &str = "linux64";
#[cfg(target_os = "macos")]
const CHROME_PLATFORM: &str = "mac-x64";

#[cfg(target_os = "windows")]
const CHROME_EXECUTABLE: &str = "chrome-headless-shell.exe";
#[cfg(not(target_os = "windows"))]
const CHROME_EXECUTABLE: &str = "chrome-headless-shell";

pub struct ChromeManager {
    chrome_dir: PathBuf,
    chrome_path: PathBuf,
}

impl ChromeManager {
    pub fn new() -> Result<Self> {
        let chrome_dir = Self::get_chrome_dir()?;
        let chrome_path = chrome_dir.join(CHROME_EXECUTABLE);
        
        Ok(Self {
            chrome_dir,
            chrome_path,
        })
    }

    /// Get the directory where Chrome should be stored
    fn get_chrome_dir() -> Result<PathBuf> {
        let app_dir = dirs::data_local_dir()
            .or_else(|| dirs::data_dir())
            .context("Failed to get application data directory")?;
        
        Ok(app_dir.join("papercraft").join("chrome"))
    }

    /// Check if Chrome is already installed and working
    pub fn is_chrome_available(&self) -> bool {
        if !self.chrome_path.exists() {
            return false;
        }

        // Test if Chrome can run
        Command::new(&self.chrome_path)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get the path to the Chrome executable
    pub fn get_chrome_path(&self) -> &Path {
        &self.chrome_path
    }

    /// Download and install Chrome Headless Shell
    pub fn download_chrome(&self) -> Result<()> {
        crate::logger::Logger::info("📥 Downloading Chrome Headless Shell...");
        
        // Create chrome directory
        fs::create_dir_all(&self.chrome_dir)
            .context("Failed to create Chrome directory")?;

        let download_url = self.get_download_url();
        let zip_path = self.chrome_dir.join("chrome-headless-shell.zip");

        // Download the zip file
        self.download_file(&download_url, &zip_path)
            .context("Failed to download Chrome Headless Shell")?;

        // Extract the zip file
        self.extract_chrome(&zip_path)
            .context("Failed to extract Chrome Headless Shell")?;

        // Clean up zip file
        let _ = fs::remove_file(&zip_path);

        // Set executable permissions on Unix-like systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.chrome_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&self.chrome_path, perms)?;
        }

        crate::logger::Logger::success("✅ Chrome Headless Shell installed successfully");
        Ok(())
    }

    /// Get the download URL for the current platform
    fn get_download_url(&self) -> String {
        format!(
            "https://storage.googleapis.com/chrome-for-testing-public/{}/{}/chrome-headless-shell-{}.zip",
            CHROME_FOR_TESTING_VERSION,
            CHROME_PLATFORM,
            CHROME_PLATFORM
        )
    }

    /// Download a file from URL to local path
    fn download_file(&self, url: &str, output_path: &Path) -> Result<()> {
        let response = reqwest::blocking::get(url)
            .context("Failed to start download")?;

        if !response.status().is_success() {
            anyhow::bail!("Download failed with status: {}", response.status());
        }

        let content = response.bytes()
            .context("Failed to read download content")?;

        fs::write(output_path, content)
            .context("Failed to write downloaded file")?;

        Ok(())
    }

    /// Extract Chrome from zip archive
    fn extract_chrome(&self, zip_path: &Path) -> Result<()> {
        let file = fs::File::open(zip_path)
            .context("Failed to open zip file")?;
        
        let mut archive = ZipArchive::new(file)
            .context("Failed to read zip archive")?;

        // Extract all files
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .context("Failed to read file from archive")?;
            
            let outpath = match file.enclosed_name() {
                Some(path) => {
                    // Strip the top-level directory from the archive
                    let components: Vec<_> = path.components().collect();
                    if components.len() > 1 {
                        let new_path: PathBuf = components[1..].iter().collect();
                        new_path
                    } else {
                        continue;
                    }
                },
                None => continue,
            };

            let outpath = self.chrome_dir.join(&outpath);

            if file.name().ends_with('/') {
                // Directory
                fs::create_dir_all(&outpath)
                    .context("Failed to create directory")?;
            } else {
                // File
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)
                            .context("Failed to create parent directory")?;
                    }
                }
                
                let mut outfile = fs::File::create(&outpath)
                    .context("Failed to create output file")?;
                
                std::io::copy(&mut file, &mut outfile)
                    .context("Failed to extract file")?;
            }
        }

        Ok(())
    }

    /// Ensure Chrome is available, download if necessary
    pub fn ensure_chrome(&self) -> Result<PathBuf> {
        if self.is_chrome_available() {
            crate::logger::Logger::verbose("✅ Chrome Headless Shell is already available");
            return Ok(self.chrome_path.clone());
        }

        crate::logger::Logger::info("🔄 Chrome Headless Shell not found, downloading...");
        self.download_chrome()?;

        if !self.is_chrome_available() {
            anyhow::bail!("Chrome Headless Shell installation failed - executable not working");
        }

        Ok(self.chrome_path.clone())
    }

    /// Get Chrome version info
    pub fn get_chrome_version(&self) -> Result<String> {
        let output = Command::new(&self.chrome_path)
            .arg("--version")
            .output()
            .context("Failed to get Chrome version")?;

        if !output.status.success() {
            anyhow::bail!("Chrome version command failed");
        }

        let version = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in Chrome version output")?;

        Ok(version.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chrome_manager_creation() {
        let manager = ChromeManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_download_url_format() {
        let manager = ChromeManager::new().unwrap();
        let url = manager.get_download_url();
        assert!(url.contains("chrome-for-testing-public"));
        assert!(url.contains(CHROME_FOR_TESTING_VERSION));
    }
}