//! WebSocket API module
//!
//! Modular request/response types and handler functions for quality analysis.
//!
//! # Structure
//!
//! ```text
//! api/
//! ├── mod.rs       - Router and re-exports
//! ├── types.rs     - ApiRequest, ApiResponse
//! ├── state.rs     - AppState
//! ├── analyze.rs   - about, analyze handlers
//! ├── curves.rs    - get_intervals, get_cdf, get_pdf handlers
//! └── histogram.rs - get_histogram handler
//! ```
//!
//! # Commands
//!
//! | Command | Handler | Module |
//! |---------|---------|--------|
//! | `about` | `handle_about` | analyze.rs |
//! | `analyze` | `handle_analyze` | analyze.rs |
//! | `get_intervals` | `handle_get_intervals` | curves.rs |
//! | `get_cdf` | `handle_get_cdf` | curves.rs |
//! | `get_pdf` | `handle_get_pdf` | curves.rs |
//! | `get_histogram` | `handle_get_histogram` | histogram.rs |

mod analyze;
mod curves;
mod histogram;
mod state;
mod types;

// Re-export types
pub use state::AppState;
pub use types::{ApiRequest, ApiResponse};

// Re-export handlers (for testing/direct use)
pub use analyze::{handle_about, handle_analyze};
pub use curves::{handle_get_cdf, handle_get_intervals, handle_get_pdf};
pub use histogram::handle_get_histogram;

use std::sync::Arc;

/// Route request to appropriate handler
pub fn handle_request(req: &ApiRequest, state: &Arc<AppState>) -> ApiResponse {
    match req.command.as_str() {
        "about" => handle_about(),
        "analyze" => handle_analyze(req, state),
        "get_intervals" => handle_get_intervals(req, state),
        "get_cdf" => handle_get_cdf(req),
        "get_pdf" => handle_get_pdf(req),
        "get_histogram" => handle_get_histogram(req, state),
        _ => ApiResponse {
            command: req.command.clone(),
            success: false,
            message: Some(format!("Unknown command: {}", req.command)),
            ..Default::default()
        },
    }
}
