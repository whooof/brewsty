use dirs::home_dir;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::Layer;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

static LOG_SENDER: std::sync::OnceLock<Sender<String>> = std::sync::OnceLock::new();
static _LOG_GUARD: std::sync::OnceLock<Arc<WorkerGuard>> = std::sync::OnceLock::new();

/// Get the log directory path: ~/.brewsty/logs/
fn get_log_dir() -> PathBuf {
    let mut path = home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".brewsty");
    path.push("logs");
    path
}

/// Initialize log capture with both UI and file logging
pub fn init_log_capture() -> Receiver<String> {
    let (tx, rx) = channel();
    LOG_SENDER
        .set(tx)
        .expect("log capture already initialized - init_log_capture() must be called exactly once");

    let capture_layer = CaptureLayer {
        sender: LOG_SENDER.get().unwrap().clone(),
    };

    #[cfg(feature = "verbose-logging")]
    let filter = LevelFilter::TRACE;

    #[cfg(not(feature = "verbose-logging"))]
    let filter = if cfg!(debug_assertions) {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    // Set up file appender with daily rotation, keep 7 days
    let log_dir = get_log_dir();
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory: {}", e);
    }

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "brewsty.log");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    _LOG_GUARD
        .set(Arc::new(guard))
        .expect("log guard already initialized");

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_level(true)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(capture_layer)
        .with(file_layer)
        .init();

    rx
}

struct CaptureLayer {
    sender: Sender<String>,
}

impl<S> Layer<S> for CaptureLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let target = metadata.target();

        if !target.starts_with("brewsty::infrastructure::brew")
            && !target.starts_with("brewsty::application")
            && !target.starts_with("brewsty::presentation")
        {
            return;
        }

        let level = *metadata.level();

        let mut visitor = LogVisitor {
            message: String::new(),
        };

        event.record(&mut visitor);

        if !visitor.message.is_empty() {
            let log_entry = format!("[{}] {}", level, visitor.message);
            let _ = self.sender.send(log_entry);
        }
    }
}

struct LogVisitor {
    message: String,
}

impl tracing::field::Visit for LogVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }
}
