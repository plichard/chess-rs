#[macro_use]
extern crate gdnative;

mod board;
mod piece;
mod utils;
mod chess;

use gdnative::prelude::{godot_init, InitHandle};

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    handle.add_class::<chess::Chess>();
}

// macros that create the entry-points of the dynamic library.
godot_init!(init);
