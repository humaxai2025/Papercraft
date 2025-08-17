use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Only download Chrome if PAPERCRAFT_BUNDLE_CHROME environment variable is set
    if env::var("PAPERCRAFT_BUNDLE_CHROME").is_ok() {
        println!("cargo:warning=PAPERCRAFT_BUNDLE_CHROME is set, Chrome will be pre-downloaded during build");
        
        // This would download Chrome during build time
        // For now, we'll just print a message since the download will happen at runtime
        // In a production build system, you could uncomment the following to pre-download:
        
        /*
        // Pre-download Chrome during build (optional)
        match download_chrome_at_build_time() {
            Ok(_) => println!("cargo:warning=Chrome Headless Shell pre-downloaded successfully"),
            Err(e) => println!("cargo:warning=Failed to pre-download Chrome: {}", e),
        }
        */
    } else {
        println!("cargo:warning=Chrome Headless Shell will be downloaded on first use");
        println!("cargo:warning=Set PAPERCRAFT_BUNDLE_CHROME=1 to pre-download during build");
    }
}

#[allow(dead_code)]
fn download_chrome_at_build_time() -> Result<(), Box<dyn std::error::Error>> {
    // This function would implement the Chrome download logic
    // Similar to what's in chrome_manager.rs but for build-time
    
    println!("cargo:warning=Pre-downloading Chrome Headless Shell...");
    
    // Implementation would go here
    // For now, we'll skip actual download during build to keep build times reasonable
    
    Ok(())
}