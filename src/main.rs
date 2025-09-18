// git add .; git commit -m "Save work before switching branches"; git checkout main
// macroquad = "0.4"

use macroquad::prelude::*;

const PLAYER_WIDTH: f32 = 40.0;
const PLAYER_HEIGHT: f32 = 50.0;
const BASE_MOVE_SPEED: f32 = 200.0;
const SPEED_BOOST: f32 = 120.0;
const GRAVITY: f32 = 800.0;
const JUMP_SPEED: f32 = 400.0;
const HIGH_JUMP_SPEED: f32 = 650.0; // NEW
const BULLET_SPEED: f32 = 500.0;
const ENEMY_WIDTH: f32 = 28.0;
const ENEMY_HEIGHT: f32 = 45.0;
const ENEMY_SPEED: f32 = 60.0;
const BONUS_SIZE: f32 = 20.0;
const POWERUP_SIZE: f32 = 20.0;
const MAX_HEALTH: i32 = 3;

#[derive(Clone, Copy)]
enum PowerUpType {
    Health,
    Speed,
    Invincibility,
    HighJump, // NEW
}

#[derive(Clone)]
struct PowerUp {
    pos: Vec2,
    kind: PowerUpType,
    collected: bool,
}

impl PowerUp {
    fn draw(&self, camera_x: f32) {
        if !self.collected {
            let color = match self.kind {
                PowerUpType::Health => PINK,
                PowerUpType::Speed => ORANGE,
                PowerUpType::Invincibility => PURPLE,
                PowerUpType::HighJump => BLUE, // NEW
            };
            draw_circle(
                self.pos.x - camera_x + POWERUP_SIZE / 2.0,
                self.pos.y + POWERUP_SIZE / 2.0,
                POWERUP_SIZE / 2.0,
                color,
            );
        }
    }
    fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, POWERUP_SIZE, POWERUP_SIZE)
    }
}

#[derive(Clone)]
struct Enemy {
    pos: Vec2,
    vel: Vec2,
    left_bound: f32,
    right_bound: f32,
    alive: bool,
    gravity: f32,
    can_be_jumped_on: bool, // NEW
}

impl Enemy {
    fn update(&mut self, dt: f32, platforms: &[Rect]) {
        if !self.alive {
            return;
        }
        self.pos.x += self.vel.x * dt;
        if self.pos.x < self.left_bound {
            self.pos.x = self.left_bound;
            self.vel.x = ENEMY_SPEED;
        }
        if self.pos.x > self.right_bound {
            self.pos.x = self.right_bound;
            self.vel.x = -ENEMY_SPEED;
        }
        self.vel.y += self.gravity * dt;
        let mut new_pos = self.pos;
        new_pos.y += self.vel.y * dt;
        let enemy_rect = Rect::new(new_pos.x, new_pos.y, ENEMY_WIDTH, ENEMY_HEIGHT);

        for platform in platforms {
            if enemy_rect.overlaps(platform) {
                if self.vel.y > 0.0 && self.pos.y + ENEMY_HEIGHT <= platform.y {
                    new_pos.y = platform.y - ENEMY_HEIGHT;
                    self.vel.y = 0.0;
                }
            }
        }
        self.pos = new_pos;
    }
    fn draw(&self, camera_x: f32) {
        if self.alive {
            let x = self.pos.x - camera_x;
            let y = self.pos.y;
            // Head
            let head_color = if self.can_be_jumped_on { LIME } else { RED };
            draw_circle(x + ENEMY_WIDTH / 2.0, y + 12.0, 10.0, head_color);
            // "Hat" for jump-on enemies
            if self.can_be_jumped_on {
                draw_rectangle(x + ENEMY_WIDTH / 2.0 - 8.0, y + 2.0, 16.0, 4.0, DARKGREEN);
            }
            // Body
            draw_rectangle(x + ENEMY_WIDTH / 2.0 - 5.0, y + 22.0, 10.0, 14.0, RED);
            // Arms
            draw_line(x + ENEMY_WIDTH / 2.0, y + 24.0, x, y + 28.0, 2.0, RED);
            draw_line(x + ENEMY_WIDTH / 2.0, y + 24.0, x + ENEMY_WIDTH, y + 28.0, 2.0, RED);
            // Legs
            draw_line(x + ENEMY_WIDTH / 2.0, y + 36.0, x + 4.0, y + ENEMY_HEIGHT, 2.0, RED);
            draw_line(x + ENEMY_WIDTH / 2.0, y + 36.0, x + ENEMY_WIDTH - 4.0, y + ENEMY_HEIGHT, 2.0, RED);
        }
    }
    fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, ENEMY_WIDTH, ENEMY_HEIGHT)
    }
}

#[derive(Clone)]
struct Bonus {
    pos: Vec2,
    collected: bool,
}

impl Bonus {
    fn draw(&self, camera_x: f32) {
        if !self.collected {
            draw_circle(
                self.pos.x - camera_x + BONUS_SIZE / 2.0,
                self.pos.y + BONUS_SIZE / 2.0,
                BONUS_SIZE / 2.0,
                GOLD,
            );
        }
    }
    fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, BONUS_SIZE, BONUS_SIZE)
    }
}

struct Bullet {
    pos: Vec2,
    vel: Vec2,
    alive: bool,
}

impl Bullet {
    fn update(&mut self, dt: f32) {
        if !self.alive {
            return;
        }
        self.pos += self.vel * dt;
        if self.pos.x < 0.0 || self.pos.x > 3000.0 || self.pos.y < 0.0 || self.pos.y > 2000.0 {
            self.alive = false;
        }
    }
    fn draw(&self, camera_x: f32) {
        if self.alive {
            draw_rectangle(self.pos.x - camera_x, self.pos.y, 10.0, 4.0, YELLOW);
        }
    }
    fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, 10.0, 4.0)
    }
}

struct Player {
    pos: Vec2,
    vel: Vec2,
    on_ground: bool,
    facing_right: bool,
    health: i32,
    score: i32,
    alive: bool,
    speed_timer: f32,
    invincible_timer: f32,
    high_jump_timer: f32, // NEW
    prev_y: f32, // NEW, for jump-on detection
}

impl Player {
    fn update(&mut self, dt: f32, platforms: &[Rect]) {
        if !self.alive {
            return;
        }
        self.prev_y = self.pos.y; // NEW
        let mut input = 0.0;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            input -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            input += 1.0;
        }
        let move_speed = BASE_MOVE_SPEED + if self.speed_timer > 0.0 { SPEED_BOOST } else { 0.0 };
        self.vel.x = input * move_speed;
        if input != 0.0 {
            self.facing_right = input > 0.0;
        }
        let jump_speed = if self.high_jump_timer > 0.0 { HIGH_JUMP_SPEED } else { JUMP_SPEED }; // NEW
        if self.on_ground
            && (is_key_pressed(KeyCode::W)
                || is_key_pressed(KeyCode::Up)
                || is_key_pressed(KeyCode::Space))
        {
            self.vel.y = -jump_speed;
            self.on_ground = false;
        }
        if self.speed_timer > 0.0 {
            self.speed_timer -= dt;
        }
        if self.invincible_timer > 0.0 {
            self.invincible_timer -= dt;
        }
        if self.high_jump_timer > 0.0 {
            self.high_jump_timer -= dt;
        }
        self.vel.y += GRAVITY * dt;
        let mut new_pos = self.pos + self.vel * dt;
        let player_rect = Rect::new(new_pos.x, new_pos.y, PLAYER_WIDTH, PLAYER_HEIGHT);

        self.on_ground = false;
        for platform in platforms {
            if player_rect.overlaps(platform) {
                if self.vel.y > 0.0 && self.pos.y + PLAYER_HEIGHT <= platform.y {
                    new_pos.y = platform.y - PLAYER_HEIGHT;
                    self.vel.y = 0.0;
                    self.on_ground = true;
                }
            }
        }
        if new_pos.y > 2000.0 {
            new_pos = vec2(100.0, 100.0);
            self.vel = Vec2::ZERO;
        }
        self.pos = new_pos;
    }

    fn draw(&self, camera_x: f32) {
        let x = self.pos.x - camera_x;
        let y = self.pos.y;
        // Head
        draw_circle(x + PLAYER_WIDTH / 2.0, y + 14.0, 12.0, YELLOW);
        // Body
        let mut body_color = BLUE;
        if !self.alive {
            body_color = GRAY;
        } else if self.invincible_timer > 0.0 {
            body_color = YELLOW;
        } else if self.high_jump_timer > 0.0 {
            body_color = BLUE;
        }
        draw_rectangle(x + PLAYER_WIDTH / 2.0 - 5.0, y + 26.0, 10.0, 16.0, body_color);
        // Arms
        draw_line(x + PLAYER_WIDTH / 2.0, y + 28.0, x, y + 35.0, 3.0, body_color);
        draw_line(x + PLAYER_WIDTH / 2.0, y + 28.0, x + PLAYER_WIDTH, y + 35.0, 3.0, body_color);
        // Legs
        draw_line(x + PLAYER_WIDTH / 2.0, y + 42.0, x + 5.0, y + PLAYER_HEIGHT, 3.0, body_color);
        draw_line(x + PLAYER_WIDTH / 2.0, y + 42.0, x + PLAYER_WIDTH - 5.0, y + PLAYER_HEIGHT, 3.0, body_color);
    }

    fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, PLAYER_WIDTH, PLAYER_HEIGHT)
    }

    fn reset(&mut self, pos: Vec2) {
        self.pos = pos;
        self.vel = Vec2::ZERO;
        self.prev_y = pos.y;
    }
}

#[derive(Clone)]
struct Level {
    platforms: Vec<Rect>,
    enemies: Vec<Enemy>,
    bonuses: Vec<Bonus>,
    powerups: Vec<PowerUp>,
    start: Vec2,
    goal_x: f32,
}

fn make_levels() -> Vec<Level> {
    vec![
        Level {
            platforms: vec![
                Rect::new(0.0, 400.0, 1000.0, 40.0),
                Rect::new(300.0, 320.0, 120.0, 20.0),
                Rect::new(600.0, 260.0, 100.0, 20.0),
                Rect::new(900.0, 350.0, 140.0, 20.0),
            ],
            enemies: vec![
                Enemy {
                    pos: vec2(320.0, 270.0),
                    vel: vec2(ENEMY_SPEED, 0.0),
                    left_bound: 300.0,
                    right_bound: 390.0,
                    alive: true,
                    gravity: GRAVITY,
                    can_be_jumped_on: true, // NEW
                },
                Enemy {
                    pos: vec2(620.0, 210.0),
                    vel: vec2(-ENEMY_SPEED, 0.0),
                    left_bound: 600.0,
                    right_bound: 690.0,
                    alive: true,
                    gravity: GRAVITY,
                    can_be_jumped_on: false, // NEW
                },
            ],
            bonuses: vec![
                Bonus { pos: vec2(340.0, 295.0), collected: false },
                Bonus { pos: vec2(650.0, 235.0), collected: false },
            ],
            powerups: vec![
                PowerUp { pos: vec2(935.0, 325.0), kind: PowerUpType::Speed, collected: false },
                PowerUp { pos: vec2(700.0, 235.0), kind: PowerUpType::HighJump, collected: false }, // NEW
            ],
            start: vec2(100.0, 100.0),
            goal_x: 1050.0,
        },
        Level {
            platforms: vec![
                Rect::new(0.0, 400.0, 1400.0, 40.0),
                Rect::new(200.0, 320.0, 120.0, 20.0),
                Rect::new(600.0, 250.0, 100.0, 20.0),
                Rect::new(1000.0, 200.0, 90.0, 20.0),
                Rect::new(1200.0, 320.0, 100.0, 20.0),
            ],
            enemies: vec![
                Enemy {
                    pos: vec2(220.0, 270.0),
                    vel: vec2(ENEMY_SPEED, 0.0),
                    left_bound: 200.0,
                    right_bound: 320.0,
                    alive: true,
                    gravity: GRAVITY,
                    can_be_jumped_on: true, // NEW
                },
                Enemy {
                    pos: vec2(620.0, 200.0),
                    vel: vec2(-ENEMY_SPEED, 0.0),
                    left_bound: 600.0,
                    right_bound: 700.0,
                    alive: true,
                    gravity: GRAVITY,
                    can_be_jumped_on: false, // NEW
                },
                Enemy {
                    pos: vec2(1020.0, 150.0),
                    vel: vec2(ENEMY_SPEED, 0.0),
                    left_bound: 1000.0,
                    right_bound: 1090.0,
                    alive: true,
                    gravity: GRAVITY,
                    can_be_jumped_on: true, // NEW
                },
            ],
            bonuses: vec![
                Bonus { pos: vec2(650.0, 225.0), collected: false },
                Bonus { pos: vec2(1250.0, 295.0), collected: false },
            ],
            powerups: vec![
                PowerUp { pos: vec2(1100.0, 175.0), kind: PowerUpType::Invincibility, collected: false },
                PowerUp { pos: vec2(1300.0, 295.0), kind: PowerUpType::HighJump, collected: false }, // NEW
            ],
            start: vec2(100.0, 100.0),
            goal_x: 1450.0,
        },
        Level {
            platforms: vec![
                Rect::new(0.0, 400.0, 1000.0, 40.0),
                Rect::new(300.0, 320.0, 120.0, 20.0),
                Rect::new(400.0, 260.0, 100.0, 20.0),
                Rect::new(400.0, 350.0, 140.0, 20.0),
            ],
            enemies: vec![
                Enemy {
                    pos: vec2(320.0, 270.0),
                    vel: vec2(ENEMY_SPEED, 0.0),
                    left_bound: 300.0,
                    right_bound: 390.0,
                    alive: true,
                    gravity: GRAVITY,
                    can_be_jumped_on: true, // NEW
                },
                Enemy {
                    pos: vec2(620.0, 210.0),
                    vel: vec2(-ENEMY_SPEED, 0.0),
                    left_bound: 600.0,
                    right_bound: 690.0,
                    alive: true,
                    gravity: GRAVITY,
                    can_be_jumped_on: false, // NEW
                },
            ],
            bonuses: vec![
                Bonus { pos: vec2(340.0, 295.0), collected: false },
                Bonus { pos: vec2(650.0, 235.0), collected: false },
            ],
            powerups: vec![
                PowerUp { pos: vec2(935.0, 325.0), kind: PowerUpType::Speed, collected: false },
                PowerUp { pos: vec2(700.0, 235.0), kind: PowerUpType::HighJump, collected: false }, // NEW
            ],
            start: vec2(100.0, 100.0),
            goal_x: 1050.0,
        }
    ]
}

#[macroquad::main("Adventure Game: Powerups & Levels")]
async fn main() {
    let mut levels = make_levels();
    let mut current_level = 0;

    let mut player = Player {
        pos: levels[0].start,
        vel: Vec2::ZERO,
        on_ground: false,
        facing_right: true,
        health: MAX_HEALTH,
        score: 0,
        alive: true,
        speed_timer: 0.0,
        invincible_timer: 0.0,
        high_jump_timer: 0.0,
        prev_y: levels[0].start.y,
    };

    let mut enemies = levels[0].enemies.clone();
    let mut bonuses = levels[0].bonuses.clone();
    let mut powerups = levels[0].powerups.clone();
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut shoot_cooldown = 0.0;

    let mut game_won = false;

    loop {
        let dt = get_frame_time();
        let platforms: &[Rect] = if current_level < levels.len() {
            &levels[current_level].platforms
        } else {
            &[] as &[Rect]
        };

        if player.alive && !game_won {
            player.update(dt, platforms);
        }

        let camera_x = player.pos.x - screen_width() / 2.0 + PLAYER_WIDTH / 2.0;

        shoot_cooldown -= dt;
        if player.alive && !game_won && (is_key_pressed(KeyCode::J) || is_key_pressed(KeyCode::Z)) && shoot_cooldown <= 0.0 {
            let up = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);
            if up {
                bullets.push(Bullet {
                    pos: vec2(
                        player.pos.x + PLAYER_WIDTH / 2.0,
                        player.pos.y,
                    ),
                    vel: vec2(0.0, -BULLET_SPEED),
                    alive: true,
                });
            } else {
                let dir = if player.facing_right { 1.0 } else { -1.0 };
                bullets.push(Bullet {
                    pos: vec2(
                        player.pos.x + PLAYER_WIDTH / 2.0 + dir * 18.0,
                        player.pos.y + PLAYER_HEIGHT / 2.0,
                    ),
                    vel: vec2(dir * BULLET_SPEED, 0.0),
                    alive: true,
                });
            }
            shoot_cooldown = 0.2;
        }

        for bullet in &mut bullets {
            bullet.update(dt);
        }
        bullets.retain(|b| b.alive);

        for enemy in &mut enemies {
            enemy.update(dt, platforms);
        }

        // Bullet-enemy collision
        for bullet in &mut bullets {
            if !bullet.alive { continue; }
            for enemy in &mut enemies {
                if enemy.alive && bullet.rect().overlaps(&enemy.rect()) {
                    enemy.alive = false;
                    bullet.alive = false;
                    if player.alive { player.score += 100; }
                }
            }
        }

        // Jump-on-enemy logic and player-enemy collision
        if player.alive && !game_won {
            let mut jumped_on_any = false;
            for enemy in &mut enemies {
                if !enemy.alive { continue; }
                let player_rect = player.rect();
                let enemy_rect = enemy.rect();
                let is_colliding = player_rect.overlaps(&enemy_rect);

                // Jump on enemy from above
                let player_was_above = player.prev_y + PLAYER_HEIGHT <= enemy.pos.y + 4.0; // fudge factor
                if enemy.can_be_jumped_on && is_colliding && player.vel.y > 0.0 && player_was_above {
                    enemy.alive = false;
                    player.vel.y = -JUMP_SPEED * 0.9; // bounce up
                    player.score += 150;
                    jumped_on_any = true;
                }
            }

            // If not jumping on any enemy, regular collision (damage)
            if !jumped_on_any && player.invincible_timer <= 0.0 {
                for enemy in &enemies {
                    if enemy.alive && player.rect().overlaps(&enemy.rect()) {
                        player.health -= 1;
                        if player.health <= 0 {
                            player.alive = false;
                        }
                        player.reset(levels[current_level].start);
                        break;
                    }
                }
            }
        }

        if player.alive && !game_won {
            for bonus in &mut bonuses {
                if !bonus.collected && player.rect().overlaps(&bonus.rect()) {
                    bonus.collected = true;
                    player.score += 50;
                }
            }
        }

        if player.alive && !game_won {
            for powerup in &mut powerups {
                if !powerup.collected && player.rect().overlaps(&powerup.rect()) {
                    powerup.collected = true;
                    match powerup.kind {
                        PowerUpType::Health => {
                            player.health = MAX_HEALTH.min(player.health + 1);
                        }
                        PowerUpType::Speed => {
                            player.speed_timer = 5.0;
                        }
                        PowerUpType::Invincibility => {
                            player.invincible_timer = 5.0;
                        }
                        PowerUpType::HighJump => { // NEW
                            player.high_jump_timer = 5.0;
                        }
                    }
                }
            }
        }

        // Level progression and win logic
        if !game_won && player.pos.x > levels[current_level].goal_x {
            current_level += 1;
            if current_level >= levels.len() {
                game_won = true;
            } else {
                player.reset(levels[current_level].start);
                enemies = levels[current_level].enemies.clone();
                bonuses = levels[current_level].bonuses.clone();
                powerups = levels[current_level].powerups.clone();
                bullets.clear();
                player.speed_timer = 0.0;
                player.invincible_timer = 0.0;
                player.high_jump_timer = 0.0;
                continue;
            }
        }

        clear_background(LIGHTGRAY);

        if game_won {
            draw_text("YOU WIN!", screen_width()/2.0-90.0, screen_height()/2.0, 56.0, DARKGREEN);
            draw_text(&format!("Final Score: {}", player.score), screen_width()/2.0-120.0, screen_height()/2.0+60.0, 40.0, BLACK);
            draw_text("Press R to Restart", screen_width()/2.0-110.0, screen_height()/2.0+120.0, 32.0, BLACK);
            if is_key_pressed(KeyCode::R) {
                game_won = false;
                current_level = 0;
                player = Player {
                    pos: levels[0].start,
                    vel: Vec2::ZERO,
                    on_ground: false,
                    facing_right: true,
                    health: MAX_HEALTH,
                    score: 0,
                    alive: true,
                    speed_timer: 0.0,
                    invincible_timer: 0.0,
                    high_jump_timer: 0.0,
                    prev_y: levels[0].start.y,
                };
                enemies = levels[0].enemies.clone();
                bonuses = levels[0].bonuses.clone();
                powerups = levels[0].powerups.clone();
                bullets.clear();
            }
            next_frame().await;
            continue;
        }

        for platform in platforms {
            draw_rectangle(platform.x - camera_x, platform.y, platform.w, platform.h, DARKGREEN);
        }
        for enemy in &enemies {
            enemy.draw(camera_x);
        }
        for bonus in &bonuses {
            bonus.draw(camera_x);
        }
        for powerup in &powerups {
            powerup.draw(camera_x);
        }
        player.draw(camera_x);
        for bullet in &bullets {
            bullet.draw(camera_x);
        }

        let health_str = format!("Health: {}/{}", player.health, MAX_HEALTH);
        draw_text(&health_str, 10.0, 30.0, 30.0, RED);
        let score_str = format!("Score: {}", player.score);
        draw_text(&score_str, 10.0, 65.0, 30.0, BLACK);
        draw_text("A/D or ←/→ to move, W/↑/Space to jump, J/Z to shoot", 10.0, 100.0, 24.0, BLACK);

        if player.speed_timer > 0.0 {
            draw_text("SPEED!", 10.0, 130.0, 28.0, ORANGE);
        }
        if player.invincible_timer > 0.0 {
            draw_text("INVINCIBLE!", 10.0, 160.0, 28.0, YELLOW);
        }
        if player.high_jump_timer > 0.0 {
            draw_text("HIGH JUMP!", 10.0, 190.0, 28.0, BLUE);
        }

        if !player.alive {
            draw_text("GAME OVER", screen_width()/2.0 - 100.0, screen_height()/2.0, 48.0, RED);
            draw_text("Press R to Restart", screen_width()/2.0 - 110.0, screen_height()/2.0 + 50.0, 32.0, BLACK);
            if is_key_pressed(KeyCode::R) {
                player = Player {
                    pos: levels[current_level].start,
                    vel: Vec2::ZERO,
                    on_ground: false,
                    facing_right: true,
                    health: MAX_HEALTH,
                    score: 0,
                    alive: true,
                    speed_timer: 0.0,
                    invincible_timer: 0.0,
                    high_jump_timer: 0.0,
                    prev_y: levels[current_level].start.y,
                };
                enemies = levels[current_level].enemies.clone();
                bonuses = levels[current_level].bonuses.clone();
                powerups = levels[current_level].powerups.clone();
                bullets.clear();
            }
        }

        next_frame().await
    }
}

