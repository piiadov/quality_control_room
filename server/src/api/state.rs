//! Application state management

use crate::config::Config;
use crate::stats::DistributionType;

/// Shared application state
pub struct AppState {
    pub config: Config,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Find best model path for given sample size
    pub fn find_model(&self, kind: DistributionType, sample_size: usize) -> Option<String> {
        let nearest = self
            .config
            .models
            .sample_sizes
            .iter()
            .min_by_key(|&&s| (s as i64 - sample_size as i64).abs())?;

        let dist_name = match kind {
            DistributionType::Beta => "Beta",
            DistributionType::Normal => "Normal",
        };

        let base = format!(
            "{}/xgb_{}_{}",
            self.config.models.models_dir, dist_name, nearest
        );

        // Try to find the latest model file
        if let Ok(entries) = std::fs::read_dir(&self.config.models.models_dir) {
            let prefix = format!("xgb_{}_{}_", dist_name, nearest);
            let mut matches: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
                .collect();

            matches.sort_by_key(|e| std::cmp::Reverse(e.file_name()));

            if let Some(entry) = matches.first() {
                return Some(entry.path().to_string_lossy().into_owned());
            }
        }

        // Fallback: try exact name with extension
        for ext in &[".ubj", ".json"] {
            let path = format!("{}{}", base, ext);
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }

        None
    }
}
