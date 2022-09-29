#![allow(dead_code)] // TODO

use eframe::epaint::Vec2;
use ui::ui::Kevboy;

mod cpu;
mod emulator;
mod mmu;
mod ui;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

const BOOT_ROM: &[u8; 32768] = include_bytes!("../blargg_tests/07-jr,jp,call,ret,rst.gb");

fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(1400.0, 720.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Kevboy-rs",
        native_options,
        Box::new(|_| Box::new(Kevboy::default())),
    );
}
