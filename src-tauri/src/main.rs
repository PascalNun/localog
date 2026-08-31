// No console window on Windows. This attribute has to live on the binary crate:
// it was in lib.rs, where it compiles without complaint and does nothing at all,
// so every Windows launch came with a console beside it that killed the
// application when closed. macOS and Linux never showed the difference.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    localog_lib::run();
}
