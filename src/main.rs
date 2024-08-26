use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Playing,
    End,
}

const SCREEN_WIDTH : i32 = 80;
const SCREEN_HEIGHT : i32 = 50;
const FRAME_DURATION : f32 = 50.0;
const SPRITE_HEIGHT : i32 = 8;
const SPRITE_WIDTH : i32 = 8;

struct Player {
    x: i32,
    y: f32,
    velocity: f32,
    flap_cycle: f32,
}

impl Player {
    fn new(x: i32, y: f32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
            flap_cycle: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(1);
        ctx.cls();

        let render_y = self.y as i32 - (SPRITE_HEIGHT / 2);

        ctx.add_sprite(
            Rect::with_size(0, render_y, SPRITE_WIDTH, SPRITE_HEIGHT),
            10,
            RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
            self.flap_cycle as usize,
        );

        ctx.set_active_console(0);
    }

    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }

        self.y += self.velocity;
        self.x += 1;
        if self.y < 0.0 {
            self.y = 0.0;
        }

        if self.flap_cycle > 3.0 {
            self.flap_cycle -= 0.4;
        } else if self.flap_cycle > 2.0 {
            self.flap_cycle -= 0.1;
        } else if self.flap_cycle > 0.0 {
            self.flap_cycle -= 0.5;
        }

        if self.flap_cycle < 0.0 {
            self.flap_cycle = 0.0
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.0;
        self.flap_cycle = 3.9;
    }
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(8, 20 - score)
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;

        for y in 0..self.gap_y - half_size {
            ctx.set(
                screen_x,
                y,
                RED,
                BLACK,
                to_cp437('|')
            );
        }

        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(
                screen_x,
                y,
                RED,
                BLACK,
                to_cp437('|')
            );
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x < self.x && player.x + SPRITE_WIDTH >= self.x;
        let player_above_gap = (player.y as i32) < self.gap_y - half_size;
        let player_below_gap = (player.y as i32) > self.gap_y + half_size;

        does_x_match && (player_above_gap || player_below_gap)
    }
}

struct State {
    player: Player,
    frame_time: f32,
    obstacle: Obstacle,
    mode: GameMode,
    score: i32,
}

impl State {
    fn new() -> Self {
        Self {
            player: Player::new(5, 25.0),
            frame_time: 0.0,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            mode: GameMode::Menu,
            score: 0,
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(5, 25.0);
        self.frame_time = 0.0;
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
        self.score = 0;
        self.mode = GameMode::Playing;
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);

        self.frame_time += ctx.frame_time_ms;

        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
        }

        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        self.player.render(ctx);

        ctx.print(0, 0, "Press SPACE to flap.");
        ctx.print(0, 1, &format!("Score: {}", self.score));

        self.obstacle.render(ctx, self.player.x);

        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }

        if self.player.y as i32 > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(0);
        ctx.print_centered(5, "Welcome to Flappy Dragon");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(0);
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(8, "(P) Play Again");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx)
        }
    }
}

embedded_resource!(DRAGON, "../resources/dragon-frames-sprite.png");

fn main() -> BError {
    link_resource!(DRAGON, "resources/dragon-frames-sprite.png");

    let context = BTermBuilder::simple80x50()
        .with_sprite_console(80, 50, 0)
        .with_font("terminal8x8.png", 8, 8)
        .with_title("Flappy Dragon")
        .with_sprite_sheet(
            SpriteSheet::new("resources/dragon-frames-sprite.png")
                .add_sprite(Rect::with_size(0, 0, 235, 170))
                .add_sprite(Rect::with_size(235, 0, 235, 170))
                .add_sprite(Rect::with_size(470, 0, 235, 170))
                .add_sprite(Rect::with_size(705, 0, 235, 170)),
        )
        .build()?;

    main_loop(context, State::new())
}
