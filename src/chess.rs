use gdnative::api::*;
use gdnative::nativescript::PropertyBuilder;
use gdnative::prelude::*;

use crate::board::Board;

#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register_builder)]
pub struct Chess {
    board: Board,
    value: i32,
    bbishop: Option<Ref<Sprite>>
}

#[methods]
impl Chess {
    fn register_builder(builder: &ClassBuilder<Self>) {
        godot_print!("Chess builder is registered");
        builder.add_property("Chess/ThisIsMyBigAssValue")
            .with_default(42)
            .with_getter(|s, _| s.value)
            .done();
    }

    fn new(_owner: &Node) -> Self {
        // godot_print!("Starting new pawn game");
        Self {
            board: Board::new_pawn_game(),
            value: 42,
            bbishop: None
        }
    }

    #[export]
    unsafe fn _ready(&mut self, owner: &Node) {
        godot_print!("Chess is ready!");
        let bbishop = owner.get_node_as::<Sprite>("BlackBishop");

        if let Some(sprite) = bbishop {
            godot_dbg!(sprite);
            self.bbishop = Some(sprite.claim());


        }
    }

    #[export]
    unsafe fn _process(&self, owner: &Node, delta: f64) {
        // godot_print!("Inside {} _process(), delta is {}", "Chess", delta);

    }
}