use crate::context::Context;
use crate::expr::Expr;
use crate::stdlib::game::GAME_STATE; // Import the global state
use minifb::Key;
use rand::Rng;
use std::time::Instant;

// ------------------------------------------------------------------
// Vector Math Helper
// ------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
    fn scale(self, s: f32) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }
    fn len_sq(self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    fn normalize(self) -> Vec2 {
        let l = self.len_sq().sqrt();
        if l > 0.0 { self.scale(1.0 / l) } else { self }
    }
    fn dist_sq(self, other: Vec2) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
    fn rotate(self, angle: f32) -> Vec2 {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Vec2::new(
            self.x * cos_a - self.y * sin_a,
            self.x * sin_a + self.y * cos_a,
        )
    }
    fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }
}

// ------------------------------------------------------------------
// Enums & Config
// ------------------------------------------------------------------
#[derive(Clone, Copy, PartialEq)]
enum Side {
    Attacker,
    Defender,
}
#[derive(Clone, Copy, PartialEq, Debug)]
enum UnitType {
    Infantry,
    Archer,
    Cavalry,
    Artillery,
}
#[derive(Clone, Copy, PartialEq)]
enum Formation {
    Line,
    Wedge,
    Loose,
    Battery,
}

struct BattleConfig {
    inf_a: usize,
    arch_a: usize,
    cav_a: usize,
    art_a: usize,
    inf_d: usize,
    arch_d: usize,
    cav_d: usize,
    art_d: usize,
}

struct BattleResult {
    winner: String,
    remaining: BattleConfig,
}

// ------------------------------------------------------------------
// Entities
// ------------------------------------------------------------------
#[derive(Clone, Copy, PartialEq, Eq)]
enum ParticleType {
    Arrow,
    Shell,
    Explosion,
    Blood,
    Smoke,
}

#[derive(Clone, Copy, PartialEq)]
struct Particle {
    p_type: ParticleType,
    start: Vec2,
    end: Vec2,
    current: Vec2,
    vel: Vec2,
    progress: f32,
    arc_h: f32,
    life: f32,
    max_life: f32,
    color: u32,
    size: f32,
}

struct Unit {
    _id: usize,
    pos: Vec2,
    vel: Vec2,
    hp: i32,
    max_hp: i32,
    damage: i32,
    range: f32,
    speed: f32,
    u_type: UnitType,
    side: Side,
    cooldown: f32,
    squad_id: usize,
    color: u32,
    facing: Vec2,
    recoil_anim: f32,
}

#[derive(Clone)]
struct Squad {
    side: Side,
    u_type: UnitType,
    members: Vec<usize>,
    target_pos: Vec2,
    center_pos: Vec2,
    formation: Formation,
    facing: Vec2, // Squad-level orientation
}

// ------------------------------------------------------------------
// Rendering Engine (Software)
// ------------------------------------------------------------------
struct Renderer<'a> {
    buffer: &'a mut Vec<u32>,
    width: usize,
    height: usize,
    shake_offset: Vec2,
}

impl<'a> Renderer<'a> {
    fn blend_color(bg: u32, fg: u32, alpha: u8) -> u32 {
        if alpha == 0 {
            return bg;
        }
        if alpha == 255 {
            return fg;
        }
        let a = alpha as u32;
        let inv_a = 255 - a;
        let r = (((fg >> 16) & 0xFF) * a + ((bg >> 16) & 0xFF) * inv_a) / 255;
        let g = (((fg >> 8) & 0xFF) * a + ((bg >> 8) & 0xFF) * inv_a) / 255;
        let b = ((fg & 0xFF) * a + (bg & 0xFF) * inv_a) / 255;
        (r << 16) | (g << 8) | b
    }

    fn put_pixel(&mut self, x: i64, y: i64, color: u32) {
        let rx = x + self.shake_offset.x as i64;
        let ry = y + self.shake_offset.y as i64;
        if rx >= 0 && rx < self.width as i64 && ry >= 0 && ry < self.height as i64 {
            self.buffer[(ry as usize) * self.width + (rx as usize)] = color;
        }
    }

    fn put_pixel_alpha(&mut self, x: i64, y: i64, color: u32, alpha: u8) {
        let rx = x + self.shake_offset.x as i64;
        let ry = y + self.shake_offset.y as i64;
        if rx >= 0 && rx < self.width as i64 && ry >= 0 && ry < self.height as i64 {
            let idx = (ry as usize) * self.width + (rx as usize);
            let bg = self.buffer[idx];
            self.buffer[idx] = Self::blend_color(bg, color, alpha);
        }
    }

    fn draw_rect_filled(&mut self, x: i64, y: i64, w: i64, h: i64, color: u32) {
        for iy in y..y + h {
            for ix in x..x + w {
                self.put_pixel(ix, iy, color);
            }
        }
    }

    fn draw_shadow(&mut self, cx: i64, cy: i64, rx: i64, ry: i64) {
        for y in -ry..=ry {
            for x in -rx..=rx {
                if (x * x * ry * ry) + (y * y * rx * rx) <= (rx * rx * ry * ry) {
                    self.put_pixel_alpha(cx + x, cy + y + 3, 0x000000, 80);
                }
            }
        }
    }

    fn draw_line(&mut self, x0: i64, y0: i64, x1: i64, y1: i64, color: u32, thickness: i64) {
        let mut x = x0;
        let mut y = y0;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            for t_offset in 0..thickness {
                self.put_pixel(x + t_offset - thickness / 2, y, color);
                self.put_pixel(x, y + t_offset - thickness / 2, color);
            }
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_circle_filled(&mut self, cx: i64, cy: i64, r: i64, color: u32, alpha: u8) {
        for y in -r..=r {
            for x in -r..=r {
                if x * x + y * y <= r * r {
                    if alpha == 255 {
                        self.put_pixel(cx + x, cy + y, color);
                    } else {
                        self.put_pixel_alpha(cx + x, cy + y, color, alpha);
                    }
                }
            }
        }
    }
}

// ------------------------------------------------------------------
// Simulation Logic
// ------------------------------------------------------------------

fn get_damage_multiplier(attacker: UnitType, defender: UnitType) -> f32 {
    match (attacker, defender) {
        (UnitType::Infantry, UnitType::Cavalry) => 3.5,
        (UnitType::Cavalry, UnitType::Archer) => 3.0,
        (UnitType::Cavalry, UnitType::Artillery) => 5.0,
        (UnitType::Archer, UnitType::Infantry) => 1.5,
        _ => 1.0,
    }
}

fn generate_terrain(width: usize, height: usize) -> Vec<u32> {
    let mut buffer = vec![0; width * height];
    let mut rng = rand::rng();
    let tile_size = 4;
    for y in (0..height).step_by(tile_size) {
        for x in (0..width).step_by(tile_size) {
            let noise = rng.random_range(0..20);
            let r = 30 + noise;
            let g = 60 + noise * 2;
            let b = 30 + noise;
            let color = (r << 16) | (g << 8) | b;
            for iy in y..y + tile_size {
                for ix in x..x + tile_size {
                    if iy < height && ix < width {
                        buffer[iy * width + ix] = color;
                    }
                }
            }
        }
    }
    buffer
}

fn run_battle_simulation(config: BattleConfig) -> BattleResult {
    // 1. Get dimensions from Global GameState
    let (width, height) = {
        let state = GAME_STATE.read().unwrap();
        (state.width, state.height)
    };

    if width == 0 || height == 0 {
        return BattleResult {
            winner: "Error".to_string(),
            remaining: config,
        };
    }

    let mut rng = rand::rng();
    let mut units: Vec<Unit> = Vec::new();
    let mut squads: Vec<Squad> = Vec::new();
    let mut particles: Vec<Particle> = Vec::new();
    let mut background_layer = generate_terrain(width, height);

    // --- Spawn Logic (Divisions) ---
    let mut make_squad = |total_count: usize, u_type: UnitType, side: Side| {
        if total_count == 0 {
            return;
        }
        // Increased squad sizes slightly since units are smaller, makes armies look denser
        let max_per_squad = match u_type {
            UnitType::Infantry => 8,
            UnitType::Archer => 6,
            UnitType::Cavalry => 4,
            UnitType::Artillery => 3,
        };
        let mut remaining = total_count;
        while remaining > 0 {
            let batch_size = remaining.min(max_per_squad);
            remaining -= batch_size;
            let squad_idx = squads.len();
            // Spawning slightly further back (100.0 vs 150.0) to maximize distance
            let cx = if side == Side::Attacker {
                100.0
            } else {
                width as f32 - 100.0
            };
            let cy = rng.random_range(100.0..(height as f32 - 100.0));

            let (hp, dmg, rng_v, spd, col, form) = match (u_type, side) {
                (UnitType::Infantry, Side::Attacker) => {
                    (100, 10, 30.0, 2.8, 0x3377EE, Formation::Line)
                }
                (UnitType::Infantry, Side::Defender) => {
                    (100, 10, 30.0, 2.8, 0xEE3333, Formation::Line)
                }
                (UnitType::Archer, Side::Attacker) => {
                    (40, 15, 240.0, 3.2, 0x77AAFF, Formation::Loose)
                }
                (UnitType::Archer, Side::Defender) => {
                    (40, 15, 240.0, 3.2, 0xFF7777, Formation::Loose)
                }
                (UnitType::Cavalry, Side::Attacker) => {
                    (160, 25, 40.0, 6.5, 0xCCCCFF, Formation::Wedge)
                }
                (UnitType::Cavalry, Side::Defender) => {
                    (160, 25, 40.0, 6.5, 0xFFCCCC, Formation::Wedge)
                }
                (UnitType::Artillery, _) => (80, 55, 450.0, 1.2, 0x333333, Formation::Battery),
            };

            let mut members = Vec::new();
            for _ in 0..batch_size {
                // Reduced random spawn jitter (20.0 instead of 30.0)
                let ox = rng.random_range(-20.0..20.0);
                let oy = rng.random_range(-20.0..20.0);
                units.push(Unit {
                    _id: units.len(),
                    pos: Vec2::new(cx + ox, cy + oy),
                    vel: Vec2::zero(),
                    hp,
                    max_hp: hp,
                    damage: dmg,
                    range: rng_v,
                    speed: spd,
                    u_type,
                    side,
                    cooldown: rng.random_range(0.0..1.0),
                    squad_id: squad_idx,
                    color: col,
                    facing: Vec2::new(if side == Side::Attacker { 1.0 } else { -1.0 }, 0.0),
                    recoil_anim: 0.0,
                });
                members.push(units.len() - 1);
            }
            squads.push(Squad {
                side,
                u_type,
                members,
                target_pos: Vec2::zero(),
                center_pos: Vec2::new(cx, cy),
                formation: form,
                facing: Vec2::new(if side == Side::Attacker { 1.0 } else { -1.0 }, 0.0),
            });
        }
    };

    let get_remaining_units = |units: &[Unit]| -> BattleConfig {
        let mut remaining = BattleConfig {
            inf_a: 0,
            arch_a: 0,
            cav_a: 0,
            art_a: 0,
            inf_d: 0,
            arch_d: 0,
            cav_d: 0,
            art_d: 0,
        };
        for u in units {
            if u.hp <= 0 {
                continue;
            }
            match (u.side, u.u_type) {
                (Side::Attacker, UnitType::Infantry) => remaining.inf_a += 1,
                (Side::Attacker, UnitType::Archer) => remaining.arch_a += 1,
                (Side::Attacker, UnitType::Cavalry) => remaining.cav_a += 1,
                (Side::Attacker, UnitType::Artillery) => remaining.art_a += 1,
                (Side::Defender, UnitType::Infantry) => remaining.inf_d += 1,
                (Side::Defender, UnitType::Archer) => remaining.arch_d += 1,
                (Side::Defender, UnitType::Cavalry) => remaining.cav_d += 1,
                (Side::Defender, UnitType::Artillery) => remaining.art_d += 1,
            }
        }
        remaining
    };

    make_squad(config.inf_a, UnitType::Infantry, Side::Attacker);
    make_squad(config.arch_a, UnitType::Archer, Side::Attacker);
    make_squad(config.cav_a, UnitType::Cavalry, Side::Attacker);
    make_squad(config.art_a, UnitType::Artillery, Side::Attacker);
    make_squad(config.inf_d, UnitType::Infantry, Side::Defender);
    make_squad(config.arch_d, UnitType::Archer, Side::Defender);
    make_squad(config.cav_d, UnitType::Cavalry, Side::Defender);
    make_squad(config.art_d, UnitType::Artillery, Side::Defender);

    let mut buffer: Vec<u32> = vec![0; width * height];
    let mut last_time = Instant::now();

    let mut victory_timer: Option<f32> = None;
    let mut winning_side: Option<Side> = None;
    let mut screenshake = 0.0;

    let mut running = true;

    while running {
        {
            let state = GAME_STATE.read().unwrap();
            if let Some(wrapper) = &state.window {
                if !wrapper.0.is_open() {
                    running = false;
                }
            } else {
                running = false;
            }
        }

        if !running {
            break;
        }

        let now = Instant::now();
        let dt = now.duration_since(last_time).as_secs_f32().min(0.05);
        last_time = now;

        screenshake = (screenshake - dt * 20.0).max(0.0);
        let shake_vec = if screenshake > 0.0 {
            Vec2::new(
                rng.random_range(-screenshake..screenshake),
                rng.random_range(-screenshake..screenshake),
            )
        } else {
            Vec2::zero()
        };
        let off_x = shake_vec.x as i64;
        let off_y = shake_vec.y as i64;

        // Draw Background
        for y in 0..height {
            for x in 0..width {
                let src_x = x as i64 - off_x;
                let src_y = y as i64 - off_y;
                if src_x >= 0 && src_x < width as i64 && src_y >= 0 && src_y < height as i64 {
                    buffer[y * width + x] =
                        background_layer[src_y as usize * width + src_x as usize];
                } else {
                    buffer[y * width + x] = 0x000000;
                }
            }
        }

        if victory_timer.is_none() {
            let atk_alive = units
                .iter()
                .filter(|u| u.side == Side::Attacker && u.hp > 0)
                .count();
            let def_alive = units
                .iter()
                .filter(|u| u.side == Side::Defender && u.hp > 0)
                .count();
            if atk_alive == 0 || def_alive == 0 {
                winning_side = Some(if atk_alive > 0 {
                    Side::Attacker
                } else {
                    Side::Defender
                });
                victory_timer = Some(2.0);
            }
        }

        if let Some(timer) = victory_timer.as_mut() {
            *timer -= dt;
            if *timer <= 0.0 {
                return BattleResult {
                    winner: match winning_side {
                        Some(Side::Attacker) => "Attacker".to_string(),
                        _ => "Defender".to_string(),
                    },
                    remaining: get_remaining_units(&units),
                };
            }
        } else {
            // --- AI & Physics Update ---
            let squads_ref = squads.clone();
            for squad in &mut squads {
                squad.members.retain(|&i| units[i].hp > 0);
                if squad.members.is_empty() {
                    continue;
                }

                let sum = squad
                    .members
                    .iter()
                    .fold(Vec2::zero(), |acc, &i| acc.add(units[i].pos));
                squad.center_pos = sum.scale(1.0 / squad.members.len() as f32);

                let mut nearest_enemy_pos = Vec2::new(width as f32 / 2.0, height as f32 / 2.0);
                let mut min_dist = f32::MAX;
                let mut enemy_found = false;

                for enemy in &squads_ref {
                    if enemy.side != squad.side && !enemy.members.is_empty() {
                        let d = squad.center_pos.dist_sq(enemy.center_pos);
                        if d < min_dist {
                            min_dist = d;
                            nearest_enemy_pos = enemy.center_pos;
                            enemy_found = true;
                        }
                    }
                }
                if !enemy_found {
                    let mut enemy_sum = Vec2::zero();
                    let mut enemy_count = 0;
                    for u in units.iter().filter(|u| u.side != squad.side && u.hp > 0) {
                        enemy_sum = enemy_sum.add(u.pos);
                        enemy_count += 1;
                    }
                    if enemy_count > 0 {
                        nearest_enemy_pos = enemy_sum.scale(1.0 / enemy_count as f32);
                        min_dist = squad.center_pos.dist_sq(nearest_enemy_pos);
                    }
                }

                let dist = min_dist.sqrt();
                let dir_to_enemy = nearest_enemy_pos.sub(squad.center_pos).normalize();
                squad.facing = squad.facing.add(dir_to_enemy.scale(0.1)).normalize();

                match squad.u_type {
                    UnitType::Archer => {
                        squad.target_pos = if dist < 150.0 {
                            squad.center_pos.sub(dir_to_enemy.scale(100.0))
                        } else if dist < 200.0 {
                            squad.center_pos
                        } else {
                            nearest_enemy_pos
                        };
                    }
                    UnitType::Artillery => {
                        squad.target_pos = if dist < 250.0 {
                            squad.center_pos
                        } else {
                            nearest_enemy_pos
                        }
                    }
                    UnitType::Cavalry => {
                        if dist > 350.0 {
                            let perp = Vec2::new(-dir_to_enemy.y, dir_to_enemy.x);
                            squad.target_pos = nearest_enemy_pos.add(perp.scale(200.0));
                        } else {
                            squad.target_pos = nearest_enemy_pos;
                        }
                    }
                    _ => squad.target_pos = nearest_enemy_pos,
                }
            }

            let mut damage_events: Vec<(usize, i32)> = Vec::new();

            for i in 0..units.len() {
                if units[i].hp <= 0 {
                    continue;
                }
                units[i].recoil_anim = (units[i].recoil_anim - dt * 5.0).max(0.0);

                let mut target_idx = None;
                let mut min_d = f32::MAX;
                for j in 0..units.len() {
                    if units[i].side != units[j].side && units[j].hp > 0 {
                        let d = units[i].pos.dist_sq(units[j].pos);
                        if d < min_d {
                            min_d = d;
                            target_idx = Some(j);
                        }
                    }
                }

                let mut force = Vec2::zero();
                let mut engaged = false;

                if let Some(t_idx) = target_idx {
                    let dist_to_target = min_d.sqrt();
                    let desired_facing = units[t_idx].pos.sub(units[i].pos).normalize();
                    units[i].facing = units[i].facing.add(desired_facing.scale(0.2)).normalize();

                    if dist_to_target <= units[i].range {
                        engaged = true;
                        if units[i].cooldown <= 0.0 {
                            units[i].recoil_anim = 1.0;
                            let dmg = (units[i].damage as f32
                                * get_damage_multiplier(units[i].u_type, units[t_idx].u_type))
                                as i32;

                            if units[i].u_type == UnitType::Archer
                                || units[i].u_type == UnitType::Artillery
                            {
                                let is_art = units[i].u_type == UnitType::Artillery;
                                let spread = Vec2::new(
                                    rng.random_range(-5.0..5.0),
                                    rng.random_range(-5.0..5.0),
                                );
                                particles.push(Particle {
                                    p_type: if is_art {
                                        ParticleType::Shell
                                    } else {
                                        ParticleType::Arrow
                                    },
                                    start: units[i].pos,
                                    end: units[t_idx].pos.add(spread),
                                    current: units[i].pos,
                                    vel: Vec2::zero(),
                                    progress: 0.0,
                                    arc_h: if is_art { 200.0 } else { 40.0 },
                                    life: 1.0,
                                    max_life: 1.0,
                                    color: if is_art { 0x111111 } else { 0xEEEEEE },
                                    size: if is_art { 4.0 } else { 2.0 },
                                });
                                if is_art {
                                    screenshake = 5.0;
                                    units[i].cooldown = 4.0;
                                    for _ in 0..5 {
                                        particles.push(Particle {
                                            p_type: ParticleType::Smoke,
                                            start: units[i].pos,
                                            end: Vec2::zero(),
                                            current: units[i].pos,
                                            vel: Vec2::new(
                                                rng.random_range(-20.0..20.0),
                                                rng.random_range(-20.0..20.0),
                                            ),
                                            progress: 0.0,
                                            arc_h: 0.0,
                                            life: 1.0,
                                            max_life: 1.0,
                                            color: 0x888888,
                                            size: 5.0,
                                        });
                                    }
                                } else {
                                    units[i].cooldown = 1.5;
                                }
                                if !is_art {
                                    damage_events.push((t_idx, dmg));
                                }
                            } else {
                                damage_events.push((t_idx, dmg));
                                units[i].cooldown = 0.8 + rng.random_range(0.0..0.2);
                                for _ in 0..4 {
                                    particles.push(Particle {
                                        p_type: ParticleType::Blood,
                                        start: units[t_idx].pos,
                                        end: Vec2::zero(),
                                        current: units[t_idx].pos,
                                        vel: Vec2::new(
                                            rng.random_range(-60.0..60.0),
                                            rng.random_range(-60.0..60.0),
                                        ),
                                        progress: 0.0,
                                        arc_h: 0.0,
                                        life: 0.4,
                                        max_life: 0.4,
                                        color: 0xAA0000,
                                        size: 2.0,
                                    });
                                }
                            }
                        }
                    }
                }

                let sq = &squads[units[i].squad_id];
                if !engaged && sq.members.len() < 3 {
                    if let Some(t_idx) = target_idx {
                        let dir = units[t_idx].pos.sub(units[i].pos).normalize();
                        force = force.add(dir.scale(units[i].speed * 80.0));
                    }
                } else if !engaged {
                    let m_idx = sq.members.iter().position(|&m| m == i).unwrap_or(0);
                    // --- TIGHTER FORMATION SPACING ---
                    // Reduced multipliers: Line 15->10, Wedge 20->14, Battery 40->25, Loose 25->16
                    let (ox, oy) = match sq.formation {
                        Formation::Line => {
                            let width = (sq.members.len() / 2).max(1);
                            let col = m_idx % width;
                            let row = m_idx / width;
                            ((col as f32 - width as f32 / 2.0) * 10.0, row as f32 * 10.0)
                        }
                        Formation::Wedge => {
                            let row = (m_idx as f32).sqrt() as usize;
                            let offset_in_row = m_idx as i32 - (row * row) as i32;
                            (
                                (offset_in_row as f32 - row as f32) * 14.0,
                                row as f32 * 14.0,
                            )
                        }
                        Formation::Battery => (
                            (m_idx as f32 * 25.0) - (sq.members.len() as f32 * 25.0 / 2.0),
                            0.0,
                        ),
                        Formation::Loose => {
                            let width = (sq.members.len() as f32).sqrt().ceil() as usize;
                            let jitter = ((i * 1337) % 10) as f32 - 5.0;
                            (
                                (m_idx % width) as f32 * 16.0 - (width as f32 * 8.0) + jitter,
                                (m_idx / width) as f32 * 16.0 + jitter,
                            )
                        }
                    };
                    let dir = sq.facing;
                    let right = Vec2::new(-dir.y, dir.x);
                    let slot = sq.target_pos.add(dir.scale(-oy)).add(right.scale(ox));
                    force = force.add(
                        slot.sub(units[i].pos)
                            .normalize()
                            .scale(units[i].speed * 80.0),
                    );
                }

                if !engaged {
                    for &other in &sq.members {
                        if other != i {
                            let d = units[i].pos.dist_sq(units[other].pos);
                            // --- SMALLER COLLISION RADIUS ---
                            // Reduced from 144 (r=12) to 64 (r=8)
                            if d < 64.0 && d > 0.01 {
                                let push = units[i]
                                    .pos
                                    .sub(units[other].pos)
                                    .normalize()
                                    .scale(600.0 / d.sqrt());
                                let clamped_push = if push.len_sq() > 250000.0 {
                                    push.normalize().scale(500.0)
                                } else {
                                    push
                                };
                                force = force.add(clamped_push);
                            }
                        }
                    }
                }

                units[i].cooldown -= dt;
                units[i].vel = units[i].vel.add(force.scale(dt)).scale(0.9);
                units[i].pos = units[i].pos.add(units[i].vel.scale(dt));
                units[i].pos.x = units[i].pos.x.clamp(20.0, width as f32 - 20.0);
                units[i].pos.y = units[i].pos.y.clamp(20.0, height as f32 - 20.0);
            }

            for (idx, dmg) in damage_events {
                units[idx].hp -= dmg;
                if units[idx].hp <= 0 {
                    let ux = units[idx].pos.x as usize;
                    let uy = units[idx].pos.y as usize;
                    for by in uy.saturating_sub(4)..uy.min(height).saturating_add(4) {
                        for bx in ux.saturating_sub(4)..ux.min(width).saturating_add(4) {
                            if by < height && bx < width {
                                let bg = background_layer[by * width + bx];
                                background_layer[by * width + bx] =
                                    Renderer::blend_color(bg, 0x550000, 100);
                            }
                        }
                    }
                }
            }

            let mut i = 0;
            while i < particles.len() {
                let mut keep = true;
                match particles[i].p_type {
                    ParticleType::Arrow | ParticleType::Shell => {
                        let speed_mult = if particles[i].p_type == ParticleType::Shell {
                            0.8
                        } else {
                            1.8
                        };
                        particles[i].progress += dt * speed_mult;
                        if rng.random_bool(0.3) {
                            let trail_color = if particles[i].p_type == ParticleType::Shell {
                                0x555555
                            } else {
                                0xDDDDDD
                            };
                            particles.push(Particle {
                                p_type: ParticleType::Smoke,
                                start: particles[i].current,
                                end: Vec2::zero(),
                                current: particles[i].current,
                                vel: Vec2::zero(),
                                progress: 0.0,
                                arc_h: 0.0,
                                life: 0.5,
                                max_life: 0.5,
                                color: trail_color,
                                size: 2.0,
                            });
                        }
                        if particles[i].progress >= 1.0 {
                            keep = false;
                            if particles[i].p_type == ParticleType::Shell {
                                screenshake = 10.0;
                                for j in 0..units.len() {
                                    if units[j].hp > 0
                                        && units[j].pos.dist_sq(particles[i].end) < 5000.0
                                    {
                                        units[j].hp -= 70;
                                    }
                                }
                                for _ in 0..30 {
                                    let col = if rng.random_bool(0.5) {
                                        0xFFAA44
                                    } else {
                                        0xAA4444
                                    };
                                    particles.push(Particle {
                                        p_type: ParticleType::Explosion,
                                        start: particles[i].end,
                                        end: Vec2::zero(),
                                        current: particles[i].end,
                                        vel: Vec2::new(
                                            rng.random_range(-150.0..150.0),
                                            rng.random_range(-150.0..150.0),
                                        ),
                                        progress: 0.0,
                                        arc_h: 0.0,
                                        life: 0.6,
                                        max_life: 0.6,
                                        color: col,
                                        size: rng.random_range(4.0..10.0),
                                    });
                                }
                                let ex = particles[i].end.x as i64;
                                let ey = particles[i].end.y as i64;
                                for sy in -15..15 {
                                    for sx in -15..15 {
                                        if sx * sx + sy * sy < 200 {
                                            let idx = ((ey + sy) as usize * width
                                                + (ex + sx) as usize)
                                                .min(width * height - 1);
                                            background_layer[idx] = Renderer::blend_color(
                                                background_layer[idx],
                                                0x111111,
                                                150,
                                            );
                                        }
                                    }
                                }
                            }
                        } else {
                            let linear = particles[i].start.add(
                                particles[i]
                                    .end
                                    .sub(particles[i].start)
                                    .scale(particles[i].progress),
                            );
                            let height_offset = particles[i].arc_h
                                * 4.0
                                * particles[i].progress
                                * (1.0 - particles[i].progress);
                            particles[i].current = Vec2::new(linear.x, linear.y - height_offset);
                        }
                    }
                    ParticleType::Blood => {
                        particles[i].life -= dt;
                        particles[i].vel = particles[i].vel.scale(0.9);
                        particles[i].current = particles[i].current.add(particles[i].vel.scale(dt));
                        if particles[i].life <= 0.0 {
                            keep = false;
                            if particles[i].current.x > 0.0
                                && particles[i].current.x < width as f32
                                && particles[i].current.y > 0.0
                                && particles[i].current.y < height as f32
                            {
                                let idx = (particles[i].current.y as usize) * width
                                    + (particles[i].current.x as usize);
                                background_layer[idx] = 0x770000;
                            }
                        }
                    }
                    ParticleType::Explosion | ParticleType::Smoke => {
                        particles[i].life -= dt;
                        particles[i].current = particles[i].current.add(particles[i].vel.scale(dt));
                        particles[i].vel = particles[i].vel.scale(0.95);
                        particles[i].current.y -= 10.0 * dt;
                        if particles[i].life <= 0.0 {
                            keep = false;
                        }
                    }
                }
                if keep {
                    i += 1;
                } else {
                    particles.swap_remove(i);
                }
            }
        }

        // --- 4. RENDERER ---
        let mut r = Renderer {
            buffer: &mut buffer,
            width,
            height,
            shake_offset: shake_vec,
        };

        for u in units.iter().filter(|u| u.hp > 0) {
            // Smaller shadows (8 / 4 instead of 12 / 7)
            let sz = if u.u_type == UnitType::Cavalry || u.u_type == UnitType::Artillery {
                8
            } else {
                4
            };
            r.draw_shadow(u.pos.x as i64, u.pos.y as i64 + 2, sz + 2, sz);
        }

        for u in units.iter().filter(|u| u.hp > 0) {
            let mut x = u.pos.x as i64;
            let mut y = u.pos.y as i64;
            let recoil = u.facing.scale(-u.recoil_anim * 3.0);
            x += recoil.x as i64;
            y += recoil.y as i64;
            let perp = Vec2::new(-u.facing.y, u.facing.x);

            // --- REDUCED DRAW SIZES (approx 60% of original) ---
            match u.u_type {
                UnitType::Infantry => {
                    // Shield reduced: offset -8->-5, size 8->5
                    let shield_pos = u.pos.add(perp.scale(-5.0)).add(u.facing.scale(3.0));
                    r.draw_rect_filled(
                        shield_pos.x as i64 - 2,
                        shield_pos.y as i64 - 2,
                        5,
                        5,
                        0x888888,
                    );

                    // Body radius 7 -> 4
                    r.draw_circle_filled(x, y, 4, u.color, 255);
                    r.draw_circle_filled(x, y - 1, 2, 0xDDDDDD, 255); // Helmet

                    // Spear length 20 -> 14
                    let thrust = 3.0 + u.recoil_anim * 10.0;
                    let spear_start = u.pos.add(perp.scale(4.0));
                    let spear_end = spear_start.add(u.facing.scale(14.0 + thrust));
                    r.draw_line(
                        spear_start.x as i64,
                        spear_start.y as i64,
                        spear_end.x as i64,
                        spear_end.y as i64,
                        0xCCCCCC,
                        1,
                    );
                }
                UnitType::Archer => {
                    // Body radius 7 -> 4
                    r.draw_circle_filled(x, y, 4, u.color, 255);
                    r.draw_circle_filled(x, y - 1, 3, 0x442211, 255); // Hood

                    // Bow reduced offsets
                    let bow_center = u.pos.add(u.facing.scale(9.0));
                    let left_tip = bow_center.add(perp.scale(-8.0)).add(u.facing.scale(-3.0));
                    let right_tip = bow_center.add(perp.scale(8.0)).add(u.facing.scale(-3.0));
                    r.draw_line(
                        left_tip.x as i64,
                        left_tip.y as i64,
                        bow_center.x as i64,
                        bow_center.y as i64,
                        0x8B4513,
                        1,
                    );
                    r.draw_line(
                        right_tip.x as i64,
                        right_tip.y as i64,
                        bow_center.x as i64,
                        bow_center.y as i64,
                        0x8B4513,
                        1,
                    );

                    if u.recoil_anim < 0.5 {
                        let arrow_tip = bow_center.add(u.facing.scale(4.0));
                        r.draw_line(x, y, arrow_tip.x as i64, arrow_tip.y as i64, 0xEEEEEE, 1);
                    }
                }
                UnitType::Cavalry => {
                    let angle = u.facing.angle();
                    let cos = angle.cos();
                    let sin = angle.sin();
                    // Horse loop reduced (-10..10, -4..4)
                    for i in -10..10 {
                        for j in -4..4 {
                            let rx = (i as f32 * cos - j as f32 * sin) + u.pos.x;
                            let ry = (i as f32 * sin + j as f32 * cos) + u.pos.y;
                            r.put_pixel(rx as i64, ry as i64, u.color);
                        }
                    }
                    // Rider radius 6 -> 4
                    r.draw_circle_filled(x, y - 5, 4, 0xFFFFFF, 255);
                    // Lance length 40 -> 26
                    let lance = u.pos.add(u.facing.scale(26.0));
                    r.draw_line(x, y - 3, lance.x as i64, lance.y as i64, 0xAAAAAA, 2);
                }
                UnitType::Artillery => {
                    // Wheels reduced
                    let w1 = u.pos.add(perp.scale(8.0));
                    let w2 = u.pos.add(perp.scale(-8.0));
                    r.draw_rect_filled(w1.x as i64 - 2, w1.y as i64 - 5, 4, 10, 0x221100);
                    r.draw_rect_filled(w2.x as i64 - 2, w2.y as i64 - 5, 4, 10, 0x221100);
                    // Body radius 10 -> 7
                    r.draw_circle_filled(x, y, 7, 0x444444, 255);
                    // Barrel reduced 30 -> 20
                    let barrel = u.pos.add(u.facing.scale(20.0 - u.recoil_anim * 8.0));
                    r.draw_line(x, y, barrel.x as i64, barrel.y as i64, 0x111111, 5);
                }
            }
            // Smaller health bar (Width 16, Height 3)
            if u.hp < u.max_hp {
                let hp_pct = u.hp as f32 / u.max_hp as f32;
                let bar_w = 16;
                let bar_x = x - 8;
                let bar_y = y - 12;
                r.draw_rect_filled(bar_x, bar_y, bar_w, 3, 0x330000);
                r.draw_rect_filled(bar_x, bar_y, (bar_w as f32 * hp_pct) as i64, 3, 0x00FF00);
            }
        }

        for p in &particles {
            let alpha = (255.0 * (p.life / p.max_life)) as u8;
            match p.p_type {
                ParticleType::Arrow => r.draw_line(
                    p.current.x as i64,
                    p.current.y as i64,
                    p.current.x as i64 - p.vel.x as i64 * 2,
                    p.current.y as i64 - p.vel.y as i64 * 2,
                    p.color,
                    2,
                ),
                ParticleType::Shell => {
                    r.draw_circle_filled(p.current.x as i64, p.current.y as i64, 4, 0x000000, 255)
                }
                ParticleType::Explosion => {
                    r.draw_circle_filled(
                        p.current.x as i64,
                        p.current.y as i64,
                        p.size as i64 * 2,
                        p.color,
                        alpha,
                    );
                }
                ParticleType::Smoke => {
                    r.draw_circle_filled(
                        p.current.x as i64,
                        p.current.y as i64,
                        p.size as i64 * 2,
                        p.color,
                        alpha / 2,
                    );
                }
                ParticleType::Blood => {
                    r.draw_rect_filled(p.current.x as i64, p.current.y as i64, 3, 3, p.color)
                }
            }
        }

        if let Some(winner) = winning_side {
            let color = if winner == Side::Attacker {
                0x4488FF
            } else {
                0xFF4444
            };
            r.draw_rect_filled(0, 0, width as i64, 40, color);
            r.draw_rect_filled(0, height as i64 - 40, width as i64, 40, color);
        }

        // 3. Update Window (Requires Lock)
        {
            let mut state = GAME_STATE.write().unwrap();
            if let Some(wrapper) = &mut state.window {
                wrapper
                    .0
                    .update_with_buffer(&buffer, width, height)
                    .unwrap();
            } else {
                break; // Window closed externally
            }
        }
    }
    BattleResult {
        winner: "Draw".to_string(),
        remaining: get_remaining_units(&units),
    }
}

pub fn register_sim_commands(game_exports: &mut std::collections::BTreeMap<Expr, Expr>) {
    game_exports.insert(Expr::sym("simulate_battle"), Expr::extern_fun(|args: &mut [Expr], ctx: &mut Context| {
        if args.len() != 8 {
             crate::stop!("simulate_battle requires exactly 8 arguments (4 attacker counts, 4 defender counts), got {}", args.len());
        }

        let evaluated_args: Vec<Expr> = args.iter().map(|a| crate::context::eval(a.clone(), ctx)).collect();
        let get_int = |i: usize| -> usize {
            match evaluated_args[i] {
                Expr::Int(n) => n as usize,
                Expr::Float(f) => f as usize,
                ref other => crate::stop!("simulate_battle argument {} must be a number, got {:?}", i, other),
            }
        };
        let result = run_battle_simulation(BattleConfig {
            inf_a: get_int(0), arch_a: get_int(1), cav_a: get_int(2), art_a: get_int(3),
            inf_d: get_int(4), arch_d: get_int(5), cav_d: get_int(6), art_d: get_int(7),
        });
        // if result.winner == "Attacker" { Expr::Int(1) } else if result.winner == "Defender" { Expr::Int(-1) } else { Expr::Int(0) }

        // Return the remaining units for both sides
        let remaining = result.remaining;
        Expr::List(vec![
            Expr::Str(result.winner),
            Expr::Int(remaining.inf_a as i64),
            Expr::Int(remaining.arch_a as i64),
            Expr::Int(remaining.cav_a as i64),
            Expr::Int(remaining.art_a as i64),
            Expr::Int(remaining.inf_d as i64),
            Expr::Int(remaining.arch_d as i64),
            Expr::Int(remaining.cav_d as i64),
            Expr::Int(remaining.art_d as i64),
        ])
    },"simulate_battle", "Visual battle sim. Args: 8 counts (A_inf, A_arc, A_cav, A_art, D_inf...)"));
}
