//! Quality Control Room WebSocket Server
//!
//! Serves WebSocket API for quality control analysis.
//! Uses trained XGBoost models for distribution parameter prediction.
//!
//! # Module Structure
//!
//! ```text
//! lib
//! ├── api/           - WebSocket API (modular handlers)
//! │   ├── types      - ApiRequest, ApiResponse
//! │   ├── state      - AppState
//! │   ├── analyze    - about, analyze handlers
//! │   ├── curves     - intervals, cdf, pdf handlers
//! │   └── histogram  - histogram handler
//! ├── config         - YAML configuration
//! ├── stats          - Statistical functions
//! └── xgb            - XGBoost FFI wrapper
//! ```

pub mod api;
pub mod config;
pub mod stats;
pub mod xgb;
