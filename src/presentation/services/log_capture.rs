use std::sync::mpsc::{Receiver, Sender, channel};
use tracing_subscriber::Layer;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

static LOG_SENDER: std::sync::OnceLock<Sender<String>> = std::sync::OnceLock::new();

pub fn init_log_capture() -> Receiver<String> {
    let (tx, rx) = channel();
    LOG_SENDER
        .set(tx)
        .expect("log capture already initialized - init_log_capture() must be called exactly once");

    let capture_layer = CaptureLayer {
        sender: LOG_SENDER.get().unwrap().clone(),
    };

    #[cfg(debug_assertions)]
    let filter = LevelFilter::TRACE;

    #[cfg(not(debug_assertions))]
    let filter = LevelFilter::DEBUG;

    tracing_subscriber::registry()
        .with(filter)
        .with(capture_layer)
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
