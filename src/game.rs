use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::channel::mpsc::{channel, unbounded, Receiver, UnboundedReceiver};
use serde::Deserialize;
use std::collections::HashMap;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{HtmlImageElement, KeyboardEvent};

use crate::{
    browser,
    engine::{self, Game, Point, Rect, Renderer},
};

#[derive(Deserialize)]
struct SheetRect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

#[derive(Deserialize)]
struct Cell {
    frame: SheetRect,
}

#[derive(Deserialize)]
pub struct Sheet {
    frames: HashMap<String, Cell>,
}

pub struct WalkTheDog {
    image: Option<HtmlImageElement>,
    sheet: Option<Sheet>,
    frame: u8,
    position: Point,
    velocity: Point,
    events: Option<UnboundedReceiver<KeyboardEvent>>,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog {
            image: None,
            sheet: None,
            frame: 0,
            position: Point { x: 0, y: 0 },
            velocity: Point { x: 0, y: 0 },
            events: None,
        }
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&mut self) -> Result<()> {
        let json = browser::fetch_json("rhb.json").await?;

        let (mut sender, receiver) = unbounded();
        self.events = Some(receiver);
        self.sheet = json.into_serde()?;
        self.image = Some(engine::load_image("rhb.png").await?);

        let onkeydown: Closure<dyn FnMut(KeyboardEvent)> =
            browser::closure_wrap(Box::new(move |keycode: KeyboardEvent| {
                log!("keycode {:?}", keycode);
                sender.start_send(keycode);
            }) as Box<dyn FnMut(KeyboardEvent)>);

        browser::window()?.set_onkeydown(Some(onkeydown.as_ref().unchecked_ref()));
        onkeydown.forget();

        Ok(())
    }

    fn update(&mut self) {
        if let Some(events) = self.events.as_mut() {
            loop {
                match events.try_next() {
                    Ok(None) => break,
                    Ok(Some(evt)) => {
                        log!("Found {:#?}", evt);
                    }
                    Err(err) => {
                        log!("Nothing found, not closed {:#?}", err);
                        break;
                    }
                }
            }
        }

        if self.frame < 23 {
            self.frame += 1;
        } else {
            self.frame = 0;
        }
    }

    fn draw(&self, renderer: &Renderer) {
        let current_sprite = (self.frame / 3) + 1;
        let frame_name = format!("Run ({}).png", current_sprite);

        let sprite = self
            .sheet
            .as_ref()
            .and_then(|sheet| sheet.frames.get(&frame_name))
            .expect("Cell not found");

        renderer.clear(&Rect {
            x: 0.0,
            y: 0.0,
            width: 600.0,
            height: 600.0,
        });
        self.image.as_ref().map(|image| {
            renderer.draw_image(
                &image,
                &Rect {
                    x: sprite.frame.x.into(),
                    y: sprite.frame.y.into(),
                    width: sprite.frame.w.into(),
                    height: sprite.frame.h.into(),
                },
                &Rect {
                    x: self.position.x.into(),
                    y: self.position.y.into(),
                    width: sprite.frame.w.into(),
                    height: sprite.frame.h.into(),
                },
            );
        });
    }
}