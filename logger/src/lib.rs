pub extern crate tracing;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

pub struct Logger {}

impl Logger {
    pub fn new() -> Self {
        // Configure a custom event formatter
        // This provides more detailed logs including level, target, thread IDs, and names.
        let format = fmt::format()
            .with_level(true) // include levels in formatted output (e.g., INFO, DEBUG)
            .with_target(true) // include the module path of the log event
            .with_thread_ids(true) // include the thread ID
            .with_thread_names(true) // include the name of the current thread if available
            .compact(); // use a more compact, single-line output format

        // Initialize tracing-subscriber
        // The RUST_LOG environment variable can be used to control verbosity.
        // For example: `RUST_LOG="info"` to see only info and higher.
        // Or `RUST_LOG="my_crate=debug,other_crate=info"` for fine-grained control.
        // If RUST_LOG is not set, it defaults to "debug" level for all targets.
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

        let subscriber_builder = fmt()
            .event_format(format) // Use the custom format
            .with_env_filter(env_filter); // Apply the environment filter

        // Attempt to set the global default subscriber.
        // This can fail if a global subscriber has already been set,
        // or if we are in a context where setting a global subscriber is not allowed.
        match subscriber_builder.try_init() {
            Ok(_) => tracing::info!("Tracing subscriber initialized successfully."),
            Err(e) => eprintln!("Failed to initialize tracing subscriber: {}", e),
        }

        tracing::info!("Logger fully initialized using tracing-subscriber. Panics will be logged.");

        Logger {}
    }
}

impl Default for Logger {
    fn default() -> Self {
        Logger::new()
    }
}
