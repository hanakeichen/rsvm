pub(crate) fn get_path_separator() -> &'static str {
    if cfg!(unix) {
        ":"
    } else if cfg!(windows) {
        ";"
    } else {
        todo!();
    }
}
