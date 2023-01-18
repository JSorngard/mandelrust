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

// pub static PRERENDERED: &[u8] =
//     include_bytes!(concat!("resources", path_sep!(), "prerendered.png"));

pub static ICON: &[u8] = include_bytes!(concat!("resources", path_sep!(), "icon.png"));
