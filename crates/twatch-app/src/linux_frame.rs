//! Linux window-frame policy.
//!
//! Mirrors the desktop-environment convention used by Electron apps: CSD
//! desktops get a frameless window with app-drawn controls, while KDE,
//! XFCE, tiling compositors, and X11 keep the native frame.

#[derive(Debug, Default)]
struct DesktopEnv {
    current: Option<String>,
    session: Option<String>,
    desktop: Option<String>,
    session_type: Option<String>,
    niri_socket: bool,
}

const CSD_DESKTOPS: &[&str] = &[
    "gnome",
    "gnome-classic",
    "gnome-flashback",
    "ubuntu",
    "unity",
    "pantheon",
    "budgie",
    "budgie-desktop",
    "niri",
];

const SSD_DESKTOPS: &[&str] = &[
    "kde",
    "plasma",
    "xfce",
    "lxqt",
    "lxde",
    "mate",
    "x-cinnamon",
    "cinnamon",
    "i3",
    "sway",
    "hyprland",
    "bspwm",
    "dwm",
    "awesome",
    "qtile",
    "regolith",
];

fn desktop_tokens(env: &DesktopEnv) -> Vec<String> {
    let raw = [env.current.as_deref(), env.session.as_deref(), env.desktop.as_deref()]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(":");
    raw.split(':')
        .map(|token| token.trim().to_ascii_lowercase())
        .filter(|token| !token.is_empty())
        .collect()
}

fn decide_frameless(env: &DesktopEnv) -> bool {
    if !cfg!(target_os = "linux") {
        return false;
    }
    let wayland =
        env.session_type.as_deref().is_some_and(|kind| kind.eq_ignore_ascii_case("wayland"));
    if !wayland {
        return false;
    }

    let tokens = desktop_tokens(env);
    // Niri is a scrolling compositor; its app header replaces the CSD.
    if env.niri_socket || tokens.iter().any(|token| token == "niri") {
        return true;
    }
    // A tiling/SSD token wins over a GNOME-family token in hybrid sessions.
    if tokens.iter().any(|token| SSD_DESKTOPS.contains(&token.as_str())) {
        return false;
    }
    tokens.iter().any(|token| CSD_DESKTOPS.contains(&token.as_str()))
}

/// True when the desktop expects the app to draw its own window frame.
pub fn prefers_frameless() -> bool {
    let env = DesktopEnv {
        current: std::env::var("XDG_CURRENT_DESKTOP").ok(),
        session: std::env::var("XDG_SESSION_DESKTOP").ok(),
        desktop: std::env::var("DESKTOP_SESSION").ok(),
        session_type: std::env::var("XDG_SESSION_TYPE").ok(),
        niri_socket: std::env::var_os("NIRI_SOCKET").is_some(),
    };
    decide_frameless(&env)
}

#[cfg(test)]
mod tests {
    use super::{DesktopEnv, decide_frameless};

    fn env(current: &str, session_type: &str) -> DesktopEnv {
        DesktopEnv {
            current: Some(current.to_owned()),
            session_type: Some(session_type.to_owned()),
            ..DesktopEnv::default()
        }
    }

    #[test]
    fn niri_wayland_is_frameless() {
        assert!(decide_frameless(&env("niri", "wayland")));
    }

    #[test]
    fn gnome_wayland_is_frameless() {
        assert!(decide_frameless(&env("ubuntu:GNOME", "wayland")));
    }

    #[test]
    fn kde_and_tiling_wayland_keep_native_frame() {
        for desktop in ["KDE", "sway", "Hyprland", "XFCE"] {
            assert!(!decide_frameless(&env(desktop, "wayland")), "{desktop}");
        }
    }

    #[test]
    fn hybrid_tiling_session_keeps_native_frame() {
        assert!(!decide_frameless(&env("Regolith:GNOME", "wayland")));
    }

    #[test]
    fn x11_and_unknown_keep_native_frame() {
        assert!(!decide_frameless(&env("GNOME", "x11")));
        assert!(!decide_frameless(&env("", "wayland")));
    }
}
