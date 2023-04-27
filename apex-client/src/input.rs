use std::sync::mpsc;

use wcore::binds::KeyCode;
use winit::event::ModifiersState;

pub struct Input {
    pub requests_input : bool,
    pub input_sender   : mpsc::Sender<()>,

    pub key       : KeyCode,
    pub modifiers : ModifiersState,
}

impl Input {
    pub fn new() -> (Self, mpsc::Receiver<()>) {
        let (tx, rx) = mpsc::channel();
        return (
            Self {
                requests_input : false,
                input_sender   : tx,

                key       : Default::default(),
                modifiers : Default::default(),
            },
            rx
        );
    }
}