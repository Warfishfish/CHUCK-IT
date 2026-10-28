//! Browser GPU budget; native rendering stays unchanged.
use std::sync::OnceLock;

pub const BALANCED_SHADOW_SIZE: usize = 2048;

fn balanced_for(browser: bool, query: &str) -> bool {
    browser && !query.trim_start_matches('?').split('&').any(|p| p == "quality=high")
}

pub fn balanced() -> bool {
    static VALUE: OnceLock<bool> = OnceLock::new();
    *VALUE.get_or_init(|| {
        #[cfg(target_arch = "wasm32")]
        let query = web_sys::window().and_then(|w| w.location().search().ok()).unwrap_or_default();
        #[cfg(not(target_arch = "wasm32"))]
        let query = String::new();
        balanced_for(cfg!(target_arch = "wasm32"), &query)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_quality_is_unchanged() {
        for query in ["", "?quality=balanced", "?quality=high"] {
            assert!(!balanced_for(false, query));
        }
    }
    #[test]
    fn browser_defaults_to_balanced_with_explicit_high_escape() {
        assert!(balanced_for(true, ""));
        assert!(balanced_for(true, "?quality=balanced"));
        assert!(balanced_for(true, "?other=quality=high"));
        assert!(!balanced_for(true, "?mode=smoko&quality=high"));
    }
}
