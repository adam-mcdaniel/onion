use crate::context::Context;
use crate::expr::Expr;
use font8x8::{BASIC_FONTS, UnicodeFonts};
use image::GenericImageView;
use image::io::Reader as ImageReader;
use minifb::{Key, Window, WindowOptions};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::BTreeMap;
use std::io::Cursor;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub fn register(ctx: &mut Context) {
    let mut game_exports = BTreeMap::new();

    game_exports.insert(
        Expr::sym("run"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 3 {
                    return Expr::Nil;
                }
                let width = crate::context::eval(args[0].clone(), ctx)
                    .as_int()
                    .unwrap_or(800) as usize;
                let height = crate::context::eval(args[1].clone(), ctx)
                    .as_int()
                    .unwrap_or(600) as usize;
                let title = crate::context::eval(args[2].clone(), ctx)
                    .as_str()
                    .unwrap_or("Onion2D")
                    .to_string();

                let (_stream, stream_handle) = match OutputStream::try_default() {
                    Ok((s, h)) => (Some(s), Some(h)),
                    Err(e) => {
                        println!(
                            "Warning: Audio init failed, sound will be disabled. {:?}",
                            e
                        );
                        (None, None)
                    }
                };

                let mut window = match Window::new(&title, width, height, WindowOptions::default())
                {
                    Ok(win) => win,
                    Err(err) => {
                        println!("Unable to create window {}", err);
                        return Expr::Nil;
                    }
                };

                window.set_target_fps(60);

                if let Some(load_fn) = ctx.resolve(&Expr::sym("load")) {
                    let call_args = vec![];
                    crate::stdlib::call_anon_fn(&load_fn, &call_args, ctx);
                }

                let mut last_frame = Instant::now();

                {
                    let mut state = GAME_STATE.write().unwrap();
                    state.resize(width, height);
                    state.audio_handle = stream_handle.clone();
                }

                while window.is_open() && !window.is_key_down(Key::Escape) {
                    let now = Instant::now();
                    let dt_secs = now.duration_since(last_frame).as_secs_f64();
                    last_frame = now;

                    crate::stdlib::game::update_input_state(&window);

                    if let Some(update_fn) = ctx.resolve(&Expr::sym("update")) {
                        let call_args = vec![Expr::Float(dt_secs)];
                        crate::stdlib::call_anon_fn(&update_fn, &call_args, ctx);
                    }

                    if let Some(draw_fn) = ctx.resolve(&Expr::sym("draw")) {
                        let call_args = vec![];
                        crate::stdlib::call_anon_fn(&draw_fn, &call_args, ctx);
                    }

                    // Blit
                    {
                        let state = GAME_STATE.read().unwrap();
                        if state.width > 0 && state.height > 0 {
                            window
                                .update_with_buffer(&state.buffer, state.width, state.height)
                                .unwrap();
                        }
                    }
                }

                Expr::Nil
            },
            "run",
            "Start the game loop",
        ),
    );

    game_exports.insert(
        Expr::sym("clear"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 1 {
                    return Expr::Nil;
                }
                let color = crate::context::eval(args[0].clone(), ctx)
                    .as_int()
                    .unwrap_or(0) as u32;
                let mut state = GAME_STATE.write().unwrap();
                for p in state.buffer.iter_mut() {
                    *p = color;
                }
                Expr::Nil
            },
            "clear",
            "Clear screen",
        ),
    );

    game_exports.insert(
        Expr::sym("rect"),
        Expr::extern_fun(
            |args, ctx| {
                // x, y, w, h, color
                if args.len() != 5 {
                    return Expr::Nil;
                }
                let x = crate::context::eval(args[0].clone(), ctx)
                    .as_int()
                    .unwrap_or(0) as i64;
                let y = crate::context::eval(args[1].clone(), ctx)
                    .as_int()
                    .unwrap_or(0) as i64;
                let w = crate::context::eval(args[2].clone(), ctx)
                    .as_int()
                    .unwrap_or(0) as i64;
                let h = crate::context::eval(args[3].clone(), ctx)
                    .as_int()
                    .unwrap_or(0) as i64;
                let color = crate::context::eval(args[4].clone(), ctx)
                    .as_int()
                    .unwrap_or(0xFFFFFF) as u32;

                let mut state = GAME_STATE.write().unwrap();
                state.draw_rect(x, y, w, h, color);
                Expr::Nil
            },
            "rect",
            "Draw rectangle",
        ),
    );

    game_exports.insert(
        Expr::sym("is_key_down"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 1 {
                    return Expr::Nil;
                }
                let key_str = crate::context::eval(args[0].clone(), ctx)
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let state = GAME_STATE.read().unwrap();
                if state.is_key_down(&key_str) {
                    Expr::Int(1)
                } else {
                    Expr::Nil
                }
            },
            "is_key_down",
            "Check key state",
        ),
    );

    game_exports.insert(
        Expr::sym("load_image"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 1 {
                    return Expr::Nil;
                }
                let path = crate::context::eval(args[0].clone(), ctx)
                    .as_str()
                    .unwrap_or("")
                    .to_string();

                let img = match ImageReader::open(&path) {
                    Ok(reader) => match reader.decode() {
                        Ok(i) => i,
                        Err(e) => {
                            println!("Failed to decode image {}: {:?}", path, e);
                            return Expr::Nil;
                        }
                    },
                    Err(e) => {
                        println!("Failed to open image {}: {:?}", path, e);
                        return Expr::Nil;
                    }
                };

                let width = img.width();
                let height = img.height();
                let mut pixels = Vec::with_capacity((width * height) as usize);

                for p in img.pixels() {
                    let r = p.2[0] as u32;
                    let g = p.2[1] as u32;
                    let b = p.2[2] as u32;
                    let a = p.2[3] as u32;
                    let color = (a << 24) | (r << 16) | (g << 8) | b;
                    pixels.push(color);
                }

                let mut state = GAME_STATE.write().unwrap();
                let id = state.next_id;
                state.next_id += 1;
                state.images.insert(
                    id,
                    GameImage {
                        width,
                        height,
                        pixels,
                    },
                );
                Expr::Int(id as i64)
            },
            "load_image",
            "Load an image from file.",
        ),
    );

    game_exports.insert(
        Expr::sym("draw_image"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 3 {
                    return Expr::Nil;
                }
                let id = crate::context::eval(args[0].clone(), ctx)
                    .as_int()
                    .unwrap_or(0) as usize;
                let x = crate::context::eval(args[1].clone(), ctx)
                    .as_int()
                    .unwrap_or(0) as i64;
                let y = crate::context::eval(args[2].clone(), ctx)
                    .as_int()
                    .unwrap_or(0) as i64;

                let mut state = GAME_STATE.write().unwrap();
                state.draw_image(id, x, y);
                Expr::Nil
            },
            "draw_image",
            "Draw an image.",
        ),
    );

    game_exports.insert(
        Expr::sym("draw_text"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 4 {
                    return Expr::Nil;
                }
                let x = crate::context::eval(args[0].clone(), ctx)
                    .as_int()
                    .unwrap_or(0) as i64;
                let y = crate::context::eval(args[1].clone(), ctx)
                    .as_int()
                    .unwrap_or(0) as i64;
                let text = crate::context::eval(args[2].clone(), ctx)
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let color = crate::context::eval(args[3].clone(), ctx)
                    .as_int()
                    .unwrap_or(0xFFFFFF) as u32;

                let mut state = GAME_STATE.write().unwrap();
                state.draw_text(x, y, &text, color);
                Expr::Nil
            },
            "draw_text",
            "Draw text to screen.",
        ),
    );

    // Game.load_sound("path.wav") -> handle
    game_exports.insert(
        Expr::sym("load_sound"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 1 {
                    return Expr::Nil;
                }
                let path = crate::context::eval(args[0].clone(), ctx)
                    .as_str()
                    .unwrap_or("")
                    .to_string();

                let data = match std::fs::read(&path) {
                    Ok(d) => d,
                    Err(e) => {
                        println!("Failed to load sound {}: {:?}", path, e);
                        return Expr::Nil;
                    }
                };

                let mut state = GAME_STATE.write().unwrap();
                let id = state.next_id;
                state.next_id += 1;
                state.sounds.insert(id, GameSound { data });
                Expr::Int(id as i64)
            },
            "load_sound",
            "Load audio file.",
        ),
    );

    game_exports.insert(
        Expr::sym("play_sound"),
        Expr::extern_fun(
            |args, ctx| {
                if args.len() != 1 {
                    return Expr::Nil;
                }
                let id = crate::context::eval(args[0].clone(), ctx)
                    .as_int()
                    .unwrap_or(0) as usize;

                let state = GAME_STATE.read().unwrap();
                if let Some(sound) = state.sounds.get(&id) {
                    if let Some(handle) = &state.audio_handle {
                        let cursor = Cursor::new(sound.data.clone());
                        if let Ok(source) = Decoder::new(cursor) {
                            if let Ok(sink) = Sink::try_new(handle) {
                                sink.append(source);
                                sink.detach();
                            } else {
                            }
                        } else {
                            println!("Failed to decode audio data");
                        }
                    }
                }
                Expr::Nil
            },
            "play_sound",
            "Play a loaded sound.",
        ),
    );

    let mod_val = Expr::Ref(Arc::new(RwLock::new(Expr::Map(game_exports))));
    ctx.define(Expr::sym("Game"), mod_val);
}

lazy_static::lazy_static! {
    static ref GAME_STATE: RwLock<GameState> = RwLock::new(GameState::new());
}

struct GameImage {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
}

struct GameSound {
    data: Vec<u8>,
}

struct GameState {
    buffer: Vec<u32>,
    width: usize,
    height: usize,
    keys_down: std::collections::HashSet<String>,
    images: BTreeMap<usize, GameImage>,
    sounds: BTreeMap<usize, GameSound>,
    audio_handle: Option<OutputStreamHandle>,
    next_id: usize,
}

impl GameState {
    fn new() -> Self {
        Self {
            buffer: vec![],
            width: 0,
            height: 0,
            keys_down: std::collections::HashSet::new(),
            images: BTreeMap::new(),
            sounds: BTreeMap::new(),
            audio_handle: None,
            next_id: 1,
        }
    }

    fn resize(&mut self, w: usize, h: usize) {
        if self.width != w || self.height != h {
            self.width = w;
            self.height = h;
            self.buffer = vec![0; w * h];
        }
    }

    fn draw_rect(&mut self, x: i64, y: i64, w: i64, h: i64, color: u32) {
        if w <= 0 || h <= 0 {
            return;
        }
        let start_x = x.max(0).min(self.width as i64) as usize;
        let start_y = y.max(0).min(self.height as i64) as usize;
        let end_x = (x + w).min(self.width as i64) as usize;
        let end_y = (y + h).min(self.height as i64) as usize;

        for cy in start_y..end_y {
            let row_offset = cy * self.width;
            for cx in start_x..end_x {
                if row_offset + cx >= self.buffer.len() {
                    continue;
                }
                self.buffer[row_offset + cx] = color;
            }
        }
    }

    fn is_key_down(&self, key: &str) -> bool {
        self.keys_down.contains(&key.to_uppercase())
    }

    fn draw_image(&mut self, id: usize, x: i64, y: i64) {
        if let Some(img_data) = self.images.get(&id) {
            let img_w = img_data.width as i64;
            let img_h = img_data.height as i64;
            let pixels = &img_data.pixels;

            let start_x = x.max(0);
            let start_y = y.max(0);
            let end_x = (x + img_w).min(self.width as i64);
            let end_y = (y + img_h).min(self.height as i64);

            if start_x >= end_x || start_y >= end_y {
                return;
            }

            for cy in start_y..end_y {
                let row_offset = (cy as usize) * self.width;
                let img_row = (cy - y) as usize;
                let img_row_offset = img_row * (img_data.width as usize);

                for cx in start_x..end_x {
                    let img_col = (cx - x) as usize;
                    let src_color = pixels[img_row_offset + img_col];
                    let alpha = (src_color >> 24) & 0xFF;

                    if alpha == 0 {
                        continue; // Fully transparent
                    } else if alpha == 255 {
                        self.buffer[row_offset + (cx as usize)] = src_color & 0xFFFFFF; // Opaque
                    } else {
                        // Alpha blend
                        let dest_color = self.buffer[row_offset + (cx as usize)];

                        let sr = (src_color >> 16) & 0xFF;
                        let sg = (src_color >> 8) & 0xFF;
                        let sb = src_color & 0xFF;

                        let dr = (dest_color >> 16) & 0xFF;
                        let dg = (dest_color >> 8) & 0xFF;
                        let db = dest_color & 0xFF;

                        let inv_a = 255 - alpha;

                        let out_r = (sr * alpha + dr * inv_a) / 255;
                        let out_g = (sg * alpha + dg * inv_a) / 255;
                        let out_b = (sb * alpha + db * inv_a) / 255;

                        self.buffer[row_offset + (cx as usize)] =
                            (out_r << 16) | (out_g << 8) | out_b;
                    }
                }
            }
        }
    }

    fn draw_text(&mut self, x: i64, y: i64, text: &str, color: u32) {
        let mut curr_x = x;
        let mut curr_y = y;

        for c in text.chars() {
            if c == '\n' {
                curr_y += 8;
                curr_x = x;
                continue;
            }
            if let Some(glyph) = BASIC_FONTS.get(c) {
                for (row_i, row) in glyph.iter().enumerate() {
                    for col_i in 0..8 {
                        if (row >> col_i) & 1 == 1 {
                            let px = curr_x + col_i as i64;
                            let py = curr_y + row_i as i64;
                            if px >= 0
                                && py >= 0
                                && px < self.width as i64
                                && py < self.height as i64
                            {
                                // Simple bounds check to avoid panic
                                if (py as usize) < self.height && (px as usize) < self.width {
                                    self.buffer[(py as usize) * self.width + (px as usize)] = color;
                                }
                            }
                        }
                    }
                }
                curr_x += 8;
            }
        }
    }
}

pub fn update_input_state(window: &Window) {
    let mut state = GAME_STATE.write().unwrap();
    state.keys_down.clear();
    if window.is_key_down(Key::A) {
        state.keys_down.insert("A".to_string());
    }
    if window.is_key_down(Key::B) {
        state.keys_down.insert("B".to_string());
    }
    if window.is_key_down(Key::C) {
        state.keys_down.insert("C".to_string());
    }
    if window.is_key_down(Key::D) {
        state.keys_down.insert("D".to_string());
    }
    if window.is_key_down(Key::E) {
        state.keys_down.insert("E".to_string());
    }
    if window.is_key_down(Key::W) {
        state.keys_down.insert("W".to_string());
    }
    if window.is_key_down(Key::S) {
        state.keys_down.insert("S".to_string());
    }
    if window.is_key_down(Key::Up) {
        state.keys_down.insert("UP".to_string());
    }
    if window.is_key_down(Key::Down) {
        state.keys_down.insert("DOWN".to_string());
    }
    if window.is_key_down(Key::Left) {
        state.keys_down.insert("LEFT".to_string());
    }
    if window.is_key_down(Key::Right) {
        state.keys_down.insert("RIGHT".to_string());
    }
    if window.is_key_down(Key::Space) {
        state.keys_down.insert("SPACE".to_string());
    }
    if window.is_key_down(Key::Enter) {
        state.keys_down.insert("ENTER".to_string());
    }
    if window.is_key_down(Key::Escape) {
        state.keys_down.insert("ESCAPE".to_string());
    }
}
