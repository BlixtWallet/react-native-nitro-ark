// Re-export the log crate for consumers of this library
pub extern crate log;

#[cfg(target_os = "android")]
use android_logger::Config;
use log::LevelFilter;
#[cfg(target_os = "ios")]
use oslog::OsLogger;

pub struct Logger {}

impl Logger {
    pub fn new() -> Self {
        #[cfg(target_os = "android")]
        {
            // Android-specific logger initialization
            android_logger::init_once(
                Config::default()
                    .with_max_level(LevelFilter::Trace) // Set the max log level
                    .with_tag("NitroArk"), // Set a custom tag for logcat
            );
            log::info!("Android logger initialized.");
        }

        #[cfg(target_os = "ios")]
        {
            OsLogger::new("com.nitro.ark")
                .level_filter(LevelFilter::Trace)
                .init()
                .unwrap();
            log::info!("oslog initialized.");
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            // Standard logger for non-Android platforms (e.g., iOS, desktop)
            // The RUST_LOG environment variable can be used to control verbosity.
            // Example: `RUST_LOG=info` or `RUST_LOG=debug`
            env_logger::builder()
                .filter_level(LevelFilter::Trace) // Default level
                .parse_default_env() // Override with RUST_LOG if set
                .init();
            log::info!("Standard (env_logger) initialized.");
        }

        Logger {}
    }
}

impl Default for Logger {
    fn default() -> Self {
        Logger::new()
    }
}
