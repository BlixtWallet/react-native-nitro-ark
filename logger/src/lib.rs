// Re-export the log crate for consumers of this library
pub extern crate log;

#[cfg(target_os = "android")]
use android_logger::Config;
use log::LevelFilter;
#[cfg(target_os = "ios")]
use oslog::OsLogger;

pub struct Logger {}

impl Logger {
    pub fn new(level: LevelFilter) -> Self {
        #[cfg(target_os = "android")]
        {
            android_logger::init_once(Config::default().with_max_level(level).with_tag("NitroArk"));
            log::info!("Android logger initialized.");
        }

        #[cfg(target_os = "ios")]
        {
            OsLogger::new("com.nitro.ark")
                .level_filter(level)
                .init()
                .unwrap();
            log::info!("oslog initialized.");
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            env_logger::builder()
                .filter_level(level)
                .parse_default_env()
                .init();
            log::info!("Standard (env_logger) initialized.");
        }

        Logger {}
    }
}

impl Default for Logger {
    fn default() -> Self {
        Logger::new(LevelFilter::Debug)
    }
}
