// Define the game configuration using the turbo::cfg! macro
turbo::cfg! {r#"
    name = "PCStone"
    version = "1.0.0"
    author = "DDX510"
    description = "A simple multiplayer game"
    [settings]
    resolution = [256, 144]
"#}

const MAX_PLAYERS: usize = 2;

const PLAYER_COLORS: [u32; MAX_PLAYERS] = [
    0xffffffff, // p1
    0xff0000ff, // p2
];

const DEAD_FRAMES: u32 = 120;
const INVULNERABLE_FRAMES: u32 = 30;
// Define the game state initialization using the turbo::init! macro
turbo::init! {
    struct GameState {
       screen: enum Screen {
        Menu,
        Game,
       },
       tick: u32,
       hit_timer: u32,
       // Game Elements
       
       // Entities
       players: Vec<Player>,
       projectiles: Vec<Projectile>
    } = {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        let [screen_w, screen_h] = resolution();
        Self {
            // Initialize all fields with default values
            screen: Screen::Menu,
            tick: 0,
            players: vec![Player {
                id: 0,
                x: (0.0 + 16.0) as f32,
                y: (screen_h / 2) as f32,
                width: 14,
                height: 16,
                health: 10,
                max_health: 10,
                score: 0,
                color: PLAYER_COLORS[0],
                speed: 2.0,
                projectile_damage: 1,
                projectile_type: ProjectileType::Basic,
                cakes: 5,
                max_cakes: 5,
            }],
            projectiles: vec![],
            hit_timer: 0,
        }
    }
}



// This is where your main game loop code goes
// The stuff in this block will run ~60x per sec
// Implement the game loop using the turbo::go! macro
turbo::go! {
    let mut state = GameState::load();

    match state.screen.clone() {
        Screen::Game => {
            draw_game_screen(&state);
            update_game_screen(&mut state);
        }
        Screen::Menu => {
            draw_menu_screen(&state);
            player_detection(&mut state);
        }
    }

    state.tick += 1;
    state.save();
}

fn map_to_grid(pixel: (usize, usize)) -> (usize, usize) {
    let grid_size = 16;
    (pixel.0 / grid_size, pixel.1 / grid_size)
}

fn map_to_pixel(grid: (usize, usize)) -> (usize, usize) {
    let grid_size = 16;
    (grid.0 * grid_size, grid.1 * grid_size)
}

fn player_detection(state: &mut GameState) {
    if gamepad(0).start.just_pressed() || gamepad(0).a.just_pressed() {
        // if state.players.len() > 1 {
            state.screen = Screen::Game;
            state.tick = 0;
        // }
    }
    for i in 1..MAX_PLAYERS {
        let i = i as u32;
        if gamepad(i).a.just_pressed() || gamepad(i).b.just_pressed() {
            if state.players.iter().position(|p| p.id == i).is_none() {
                let mut player = state.players[0].clone();
                player.id = i;
                player.color = PLAYER_COLORS[i as usize];
                player.x = 240.0;
                state.players.push(player)
            }
        }
    }
}

fn draw_menu_screen(state: &GameState) {
    clear(0x000333ff);
    let [screen_w, screen_h] = canvas_size!();
    let screen_w = screen_w as i32;
    let screen_h = screen_h as i32;
    let center = screen_w / 2;

    // Draw the title
    text! ("PCStone", x = 110, y = 20, color = 0xffffffff);
    if state.tick % 60 < 30 {
        text!("Press Start to Play", x = 80, y = 100, color = 0xffffffff);
    }

    let moving = state.tick % 256;

    // Draw the cat
    sprite!("cat", x = moving as i32, y = 72, fps= 6);

    // Show players who joined
    let num_players = state.players.len();
    let left = center - ((num_players as i32 * 52) / 2);
    for (i, player) in state.players.iter().enumerate() {
        rect!(
            h = 14,
            w = 50,
            x = left + (i as i32 * 52),
            y = screen_h - 16,
            color = if player.color == 0xffffffff {
                0x000333ff
            } else {
                player.color
            },
            border_radius = 2,
        );
        text!(
            &format!("P{} joined", player.id + 1),
            font = Font::M,
            x = left + 4 + (i as i32 * (52)),
            y = screen_h - 12
        );
    }
}

fn draw_game_screen(state: &GameState) {
    clear(0x000333ff);
    let [screen_w, screen_h] = canvas_size!();
    // background
    for i in 0..16 {
        for j in 0..9 {
            let x = i * 16;
            let y = j * 16;
            sprite!("grass", x = x, y = y);
        }
    }

    rect!(x = 0, y = 0, w = 256, h = screen_h / 3, color = 0x87CEEBFF);

    let moving_x = (state.tick % 256) as i32;

    sprite!("cloud", x = moving_x, y = 5, scale_x=0.15, scale_y=0.15);


    // Draw the players
    let len = state.players.len();
    for i in 0..len {
        let player = &state.players[i];
        // crate::println!("{} {} {}", player.x, player.y, player.id.to_string());
        // if player.health > 0 {
        //     draw_player(&player, len > 1);
        // }
        draw_player(&player, len > 1);
    }

    // Draw projectiles
    for projectile in &state.projectiles {
        draw_projectile(projectile);
    }


    // Game over text
    let is_game_over = state.players.iter().any(|p| p.health <= 0);
    if is_game_over {
        draw_game_over(state, screen_w, screen_h);
    }

}

fn update_game_screen(state: &mut GameState) {
    let [screen_w, screen_h] = resolution();
    let is_game_over = state.players.iter().any(|p| p.health <= 0);
    if is_game_over {
        // Restart
        if state.hit_timer == 0 && (gamepad(0).start.just_pressed() || gamepad(0).a.just_pressed())
        {
            let mut next_state = GameState::new();
            next_state.players = state
                .players
                .iter()
                .cloned()
                .map(|p| Player {
                    id: p.id,
                    color: PLAYER_COLORS[p.id as usize],
                    x: if p.id == 0 { 16.0 } else { 240.0 },
                    ..next_state.players[0].clone()
                })
                .collect();
            *state = next_state;
        }
    } else {
        for i in 1..MAX_PLAYERS {
            let i = i as u32;
            if gamepad(i).a.just_pressed() || gamepad(i).b.just_pressed() {
                if state.players.iter().position(|p| p.id == i).is_none() {
                    let mut player = GameState::new().players[0].clone();
                    player.id = i;
                    player.x = 240.0;
                    player.color = PLAYER_COLORS[i as usize];
                    state.players.push(player)
                }
            }
        }
        for player in &mut state.players {
            if state.tick % 50 == 0 {
                player.cakes = (player.cakes + 1).min(player.max_cakes);
            }

            let player_speed = player.speed;
            let i = player.id;
            // Get player gamepad
            let gp = gamepad(i as u32);
            // Player movement handling
            if gp.up.pressed() {
                // Move up
                player.y = (player.y - player_speed).max(0.0);
            }
            if gp.down.pressed() {
                // Move down
                player.y = (player.y + player_speed).min((screen_h - player.height) as f32);
            }
            if gp.left.pressed() {
                // Move left
                player.x = (player.x - player_speed).max(0.0);
            }
            if gp.right.pressed() {
                // Move right
                player.x = (player.x + player_speed).min((screen_w - player.width) as f32);
            }
            

            let angle = if i == 0 { 0.0 } else { -180.0 };
            // Shooting projectiles
            if( gp.start.just_pressed() || gp.a.just_pressed() || gp.b.just_pressed()) && player.cakes != 0 {
                let mut bonus_damage = 0;
                player.cakes -= 1;
                state.projectiles.push(Projectile {
                    x: player.x + ((player.width / 2) as f32) - 2.0,
                    y: player.y,
                    width: 6,
                    height: 7,
                    velocity: 5.0,
                    angle: angle,
                    damage: player.projectile_damage + bonus_damage,
                    projectile_type: player.projectile_type,
                    projectile_owner: player.id as i32,
                    ttl: None,
                });
            }
        }
    }

    // Handle collisions between enemy projectiles and the player
    for i in 0..state.players.len() {
        let player = &mut state.players[i];
        state.projectiles.retain(|projectile| {
            let mut projectile_active = true;
            if projectile.projectile_owner == player.id as i32{
                return projectile_active;
            }
            let did_collide = check_collision(
                projectile.x,
                projectile.y,
                projectile.width,
                projectile.height,
                player.x,
                player.y,
                player.width,
                player.height,
            );
            if did_collide && state.hit_timer == 0 && player.health > 0 {
                let prev_hp = player.health;
                player.health = player.health.saturating_sub(projectile.damage);
                state.hit_timer = match (prev_hp, player.health) {
                    (prev, 0) if prev > 0 => DEAD_FRAMES,         // just died
                    (_, curr) if curr > 0 => INVULNERABLE_FRAMES, // damaged
                    _ => state.hit_timer,                         // been dead
                };
                projectile_active = false // Remove the projectile on collision
            }

            projectile_active
        });
    }

    // Update projectiles
    for projectile in &mut state.projectiles {
        // projectile.y -= projectile.velocity;
        let radian_angle = projectile.angle.to_radians();
        projectile.x += projectile.velocity * radian_angle.cos();
        projectile.y += projectile.velocity * radian_angle.sin();

        if let Some(ttl) = &mut projectile.ttl {
            *ttl = ttl.saturating_sub(1);
        }
    }

    // Remove expired and out-of-bounds projectiles
    state.projectiles.retain(|projectile| {
        let is_alive = projectile.ttl.map_or(true, |ttl| ttl > 0);
        let is_in_bounds = !(projectile.y < -(projectile.height as f32)
            || projectile.x < -(projectile.width as f32)
            || projectile.x > screen_w as f32
            || projectile.y > screen_h as f32);
        is_alive && is_in_bounds
    });

    state.hit_timer = state.hit_timer.saturating_sub(1);
}

fn draw_player(player: &Player, show_number: bool) {
    // draw hp bar
    let x = player.x as i32;
    let y = player.y as i32;
    rect!(
        color = 0x333333ff,
        w = 10,
        h = 2,
        x = x + (player.width / 2) as i32 - 3,
        y = y - 6
    );
    let percent_hp = player.health as f32 / player.max_health as f32;
    let color = match percent_hp {
        n if n <= 0.25 => 0xff0000ff,
        n if n <= 0.5 => 0xff9900ff,
        _ => 0x00ff00ff,
    };

    rect!(
        color = color,
        w = ((player.health as f32 / player.max_health as f32) * 10.) as u32,
        h = 2,
        x = x + (player.width / 2) as i32 - 3,
        y = y - 6
    );


    // draw bullet bar
    rect!(
        color = 0x333333ff,
        w = 10,
        h = 2,
        x = x + (player.width / 2) as i32 - 3,
        y = y - 3
    );

    // let percent_bullet = player.cakes as f32 / player.max_cakes as f32;
    let color = 0xfffee0ff;
    rect!(
        color = color,
        w = ((player.cakes as f32 / player.max_cakes as f32) * 10.) as u32,
        h = 2,
        x = x + (player.width / 2) as i32 - 3,
        y = y - 3
    );

    // draw player character
    let curr_player= player.id;
    let curr_sprite = if curr_player == 0 {
        "cat"
    } else {
        "black_cat"
    };

    // sprite!("cat", x = 1, y = 72, fps= 6);
    let curr_x = player.x as i32;
    let curr_y = player.y as i32;
    
    sprite!(curr_sprite,x = curr_x, y = curr_y, fps = 6);
    
    if show_number {
        text!(
            &format!("{}", player.id + 1),
            x = player.x as i32 + 8,
            y = player.y as i32 + 24,
            font = Font::S,
        );
    }
}

fn draw_projectile(projectile: &Projectile) {
    match projectile.projectile_type {
        // ProjectileType::Splatter => {
        //     sprite!(
        //         "projectile_ketchup",
        //         x = projectile.x as i32,
        //         y = projectile.y as i32
        //     );
        // }
        // ProjectileType::Fragment => {
        //     let color = 0xff0000ff;
        //     ellipse!(
        //         x = projectile.x as i32,
        //         y = projectile.y as i32,
        //         w = projectile.width,
        //         h = projectile.height,
        //         color = color
        //     );
        // }
        _ => {
            // let color = 0xffff00ff;
            // ellipse!(
            //     x = projectile.x as i32,
            //     y = projectile.y as i32,
            //     w = projectile.width,
            //     h = projectile.height,
            //     color = color
            // );
            if projectile.projectile_owner == 0 {
                sprite!("my_cake", x = projectile.x as i32, y = projectile.y as i32, scale_x=0.7, scale_y=0.7);
            } else {
                sprite!("cake", x = projectile.x as i32, y = projectile.y as i32, scale_x=0.7, scale_y=0.7);
            }
        }
    }
}

fn draw_game_over(state: &GameState, screen_w: u32, screen_h: u32) {
    text!(
        "GAME OVER",
        x = (screen_w as i32 / 2) - 32,
        y = (screen_h as i32 / 2) - 4,
        font = Font::L
    );

    let winner = state.players.iter().find(|p| p.health > 0);
    if let Some(winner) = winner {
        text!(
            &format!("Player {} wins!", winner.id + 1),
            x = (screen_w as i32 / 2) - 32,
            y = (screen_h as i32 / 2) + 16,
            font = Font::M
        );
    } else {
        text!(
            "Draw!",
            x = (screen_w as i32 / 2) - 32,
            y = (screen_h as i32 / 2) + 16,
            font = Font::M
        );
    }
    if state.hit_timer == 0 {
        if state.tick / 4 % 8 < 4 {
            text!(
                "PRESS START",
                x = (screen_w as i32 / 2) - 24,
                y = (screen_h as i32 / 2) - 4 + 16,
                font = Font::M
            );
        }
    }
}

// Function to check collision between two rectangular objects
#[rustfmt::skip]
fn check_collision(x1: f32, y1: f32, w1: u32, h1: u32, x2: f32, y2: f32, w2: u32, h2: u32) -> bool {
    let x1 = x1 as i32;
    let y1 = y1 as i32;
    let w1 = w1 as i32;
    let h1 = h1 as i32;
    let x2 = x2 as i32;
    let y2 = y2 as i32;
    let w2 = w2 as i32;
    let h2 = h2 as i32;
    x1 < x2 + w2 && x1 + w1 > x2 &&
    y1 < y2 + h2 && y1 + h1 > y2
}


#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
struct Player {
    id: u32,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    score: u32,
    color: u32,
    health: u32,
    max_health: u32,
    speed: f32,
    projectile_damage: u32,
    projectile_type: ProjectileType,
    cakes: u32,
    max_cakes: u32,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
// Struct for Projectiles shot by the player
struct Projectile {
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    velocity: f32,
    angle: f32,
    damage: u32,
    projectile_type: ProjectileType,
    projectile_owner: i32,
    ttl: Option<u32>,
}

#[derive(Debug, Copy, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
enum ProjectileType {
    Basic,
    // Splatter,
    // Fragment,
    // Laser,
    // Bomb,
}

// #[derive(Debug, Copy, Clone, BorshDeserialize, BorshSerialize, PartialEq)]
// enum ProjectileOwner {
//     Opponent,
//     Own,
// }
