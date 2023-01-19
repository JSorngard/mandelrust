#[cfg(not(windows))]
macro_rules! path_sep {
    () => {
        "/"
    };
}

#[cfg(windows)]
macro_rules! path_sep {
    () => {
        r"\"
    };
}

pub static ICON: &[u8] = include_bytes!(concat!("resources", path_sep!(), "icon.png"));
pub static RENDERING_IN_PROGRESS: &[u8] =
    include_bytes!(concat!("resources", path_sep!(), "rendering.png"));
