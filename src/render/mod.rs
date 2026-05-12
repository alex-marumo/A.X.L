mod markdown;
mod stream;

pub use self::markdown::{MarkdownRender, RenderOptions};
use self::stream::{markdown_stream, raw_stream};

pub struct JsonRender;

use crate::utils::{error_text, pretty_error, AbortSignal, IS_STDOUT_TERMINAL};
use crate::{client::SseEvent, config::GlobalConfig};

use anyhow::Result;
use tokio::sync::mpsc::UnboundedReceiver;

pub async fn render_stream(
    rx: UnboundedReceiver<SseEvent>,
    config: &GlobalConfig,
    abort_signal: AbortSignal,
) -> Result<()> {
    let ret = if *IS_STDOUT_TERMINAL && config.read().highlight {
        let render_options = config.read().render_options()?;
        let mut render = MarkdownRender::init(render_options)?;
        markdown_stream(rx, &mut render, &abort_signal).await
    } else {
        raw_stream(rx, &abort_signal).await
    };
    ret.map_err(|err| err.context("Failed to reader stream"))
}

pub fn render_error(err: anyhow::Error) {
    eprintln!("{}", error_text(&pretty_error(&err)));
}

impl JsonRender {
    pub fn render(text: &str) -> String {
        serde_json::json!({
            "response": text,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })
        .to_string()
    }
    pub fn render_error(err: anyhow::Error) -> String {
        serde_json::json!({
            "error": format!("{:#}", pretty_error(&err)),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })
        .to_string()
    }
}
