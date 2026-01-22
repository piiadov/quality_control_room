//! Quality Control Room WebSocket Server
//!
//! Serves WebSocket API for quality control analysis.
//! Uses trained XGBoost models for distribution parameter prediction.

pub mod api;
pub mod config;
pub mod stats;
pub mod xgb;
