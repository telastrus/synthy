use midir::MidiInput;
use midir::MidiInputConnection;
use std::sync::{Arc, Mutex};

use ggez::graphics::screen_coordinates;
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::nalgebra as na;

use crate::assets::Assets;

pub type BaseKeyboard = (SpriteBatch, SpriteBatch);

// Hard-coding the layout for now.
// TODO: be smart about this.
pub static LAYOUT: &str =
    "WBWWBWBWWBWBWBWWBWBWWBWBWBWWBWBWWBWBWBWWBWBWWBWBWBWWBWBWWBWBWBWWBWBWWBWBWBWWBWBWWBWBWBWW";

#[derive(Copy, Clone)]
pub enum KeyType {
    WHITE,
    BLACK,
}

#[derive(Copy, Clone)]
pub struct Key {
    pub key_type: KeyType,
    pub offset: ggez::nalgebra::Point2<f32>,
}

pub struct Keyboard {
    // We need to keep a handle to this, otherwise it'll get dropped and our thread with it.
    #[allow(dead_code)]
    conn: Option<MidiInputConnection<()>>,
    active_keys: Arc<Mutex<[bool; 88]>>,
    assets: Arc<Assets>,
    active_sprites: BaseKeyboard,
}

impl Keyboard {
    pub fn new(assets: Arc<Assets>) -> Self {
        let active_keys = Arc::new(Mutex::new([false; 88]));
        let active_sprites = (
            SpriteBatch::new(assets.white_key_active.clone()),
            SpriteBatch::new(assets.black_key_active.clone()),
        );
        match MidiInput::new("synthy reader") {
            Ok(midi_in) => {
                let ports = midi_in.ports();
                let keys = active_keys.clone();
                let conn = match ports.iter().last() {
                    Some(port) => {
                        println!("Connected to {:?}", midi_in.port_name(port));
                        midi_in
                            .connect(
                                port,
                                "synthy-read-input",
                                move |_, message, _| {
                                    let mut data = keys.lock().unwrap();
                                    data[(message[1] - 21) as usize] = message[0] == 144;
                                },
                                (),
                            )
                            .ok()
                    }
                    None => {
                        println!("Unable to connect to a keyboard!");
                        None
                    }
                };
                Self {
                    conn,
                    active_keys,
                    assets,
                    active_sprites,
                }
            }
            Err(e) => {
                println!("Unable to begin establishing a key: {:?}", e);
                Self {
                    conn: None,
                    active_keys,
                    assets,
                    active_sprites,
                }
            }
        }
    }

    pub fn draw_piano<T: Into<ggez::graphics::DrawParam>>(
        &mut self,
        ctx: &mut ggez::Context,
        params: T,
    ) {
        let assets = &self.assets;
        let rect = screen_coordinates(ctx);
        let width = assets.white_key.width() * 52;
        let height = assets.white_key.height() as f32;
        let (white, black) = &assets.keyboard;
        let (wa, ba) = &mut self.active_sprites;
        wa.clear();
        ba.clear();
        let keys = self.active_keys.lock().unwrap();
        for i in 0..88 {
            if keys[i] {
                let props: Key = assets.keymap[i];
                let c = (props.offset,);
                match props.key_type {
                    KeyType::WHITE => wa.add(c),
                    KeyType::BLACK => ba.add(c),
                };
            }
        }
        let mut p: ggez::graphics::DrawParam = params.into();
        p.scale = na::Vector2::new(rect.w / (width as f32), rect.h * 0.15 / height).into();
        ggez::graphics::draw(ctx, white, p.clone()).unwrap();
        ggez::graphics::draw(ctx, wa, p.clone()).unwrap();
        ggez::graphics::draw(ctx, black, p.clone()).unwrap();
        ggez::graphics::draw(ctx, ba, p).unwrap();
    }
}
