//! Minimal X11 helpers for Linux (pointer state and foreground window).

use x11rb::protocol::xproto::AtomEnum;

pub fn primary_button_down() -> bool {
    let Some((conn, screen)) = connect() else {
        return false;
    };
    let root = conn.setup().roots[screen].root;
    let reply = match conn.query_pointer(root) {
        Ok(r) => r.reply(),
        Err(_) => return false,
    };
    match reply {
        Ok(p) => (p.mask & (1 << 8)) != 0,
        Err(_) => false,
    }
}

pub fn foreground_pid() -> u32 {
    let Some((conn, screen)) = connect() else {
        return 0;
    };
    let root = conn.setup().roots[screen].root;
    let Some(net_active) = atom(&conn, b"_NET_ACTIVE_WINDOW") else {
        return 0;
    };
    let Ok(prop) = conn
        .get_property(
            false,
            root,
            net_active,
            AtomEnum::WINDOW as u32,
            0,
            1,
        )
        .and_then(|c| c.reply())
    else {
        return 0;
    };
    let window = prop.value32().and_then(|mut v| v.next()).unwrap_or(0);
    if window == 0 {
        return 0;
    }
    let Some(net_pid) = atom(&conn, b"_NET_WM_PID") else {
        return 0;
    };
    let Ok(prop) = conn
        .get_property(
            false,
            window,
            net_pid,
            AtomEnum::CARDINAL as u32,
            0,
            1,
        )
        .and_then(|c| c.reply())
    else {
        return 0;
    };
    prop.value32().and_then(|mut v| v.next()).unwrap_or(0)
}

fn atom(conn: &x11rb::rust_connection::RustConnection, name: &[u8]) -> Option<u32> {
    conn.intern_atom(false, name)
        .ok()?
        .reply()
        .ok()
        .map(|r| r.atom)
}

fn connect() -> Option<(x11rb::rust_connection::RustConnection, usize)> {
    if std::env::var_os("DISPLAY").is_none() {
        return None;
    }
    let (conn, screen) = x11rb::connect(None).ok()?;
    Some((conn, screen))
}
