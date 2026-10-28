//! DOM owns pointer capture: requests must happen inside a trusted browser click.
pub fn set_playing(playing: bool) {
    if let Some(canvas) = web_sys::window().and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("bbq")) {
        let _ = canvas.set_attribute("data-playing", if playing { "true" } else { "false" });
    }
}
pub fn locked() -> bool {
    web_sys::window().and_then(|w| w.document())
        .and_then(|d| d.pointer_lock_element()).is_some_and(|e| e.id() == "bbq")
}
