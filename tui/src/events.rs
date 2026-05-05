pub(crate) enum TuiEvent {
    Input(Vec<u8>),
    Resize { cols: u16, rows: u16 },
}
