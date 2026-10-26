//! Conservative session detection; an explicit desktop wins over inherited sockets.
pub(crate) fn tiling_session(
    desktop: Option<&str>,
    hyprland: bool,
    sway: bool,
    niri: bool,
) -> bool {
    match desktop.filter(|value| !value.trim().is_empty()) {
        Some(value) => value.split(':').any(|name| {
            ["hyprland", "sway", "niri"]
                .iter()
                .any(|known| name.trim().eq_ignore_ascii_case(known))
        }),
        None => hyprland || sway || niri,
    }
}
pub(crate) fn current_session_is_tiled() -> bool {
    if !cfg!(target_os = "linux") {
        return false;
    }
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").ok();
    let present = |name| std::env::var(name).is_ok_and(|value| !value.is_empty());
    tiling_session(
        desktop.as_deref(),
        present("HYPRLAND_INSTANCE_SIGNATURE"),
        present("SWAYSOCK"),
        present("NIRI_SOCKET"),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_known_sessions_and_keeps_unknown_desktops_conservative() {
        assert!(tiling_session(Some("Hyprland"), false, false, false));
        assert!(tiling_session(Some("sway:wlroots"), false, false, false));
        assert!(tiling_session(Some("NIRI"), false, false, false));
        assert!(tiling_session(None, true, false, false));
        assert!(!tiling_session(Some("GNOME"), true, false, false));
        assert!(!tiling_session(Some("KDE"), false, false, false));
        assert!(!tiling_session(None, false, false, false));
    }
}
