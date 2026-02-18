#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    veteran_desktop::logger::log("=== Backend starting ===");
    veteran_desktop::run();
}
