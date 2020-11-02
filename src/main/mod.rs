use noise::{NoiseFn, Perlin};
use tcod::colors::*;
use tcod::console::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

// actual size of the window
const SCREEN_WIDTH: i32 = 180;
const SCREEN_HEIGHT: i32 = 90;

// size of the map
const MAP_WIDTH: i32 = 180;
const MAP_HEIGHT: i32 = 90;
const GAME_SEED: f64 = 1.5;

const LIMIT_FPS: i32 = 20; // 20 frames-per-second maximum

const COLOR_MOUNTAIN: Color = Color {
    r: 244,
    g: 251,
    b: 252,
};
const COLOR_HILL: Color = Color {
    r: 214,
    g: 163,
    b: 110,
};
const COLOR_SEA: Color = Color {
    r: 127,
    g: 191,
    b: 191,
};
const COLOR_FOREST: Color = Color {
    r: 127,
    g: 191,
    b: 127,
};
const COLOR_PLAINS: Color = Color {
    r: 161,
    g: 214,
    b: 110,
};
// const COLOR_TOWN: Color = Color { r: 161, g: 140, b: 118 };

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic; // default FOV algorithm
const FOV_LIGHT_WALLS: bool = true; // light walls or not
const TORCH_RADIUS: i32 = 10;

struct Tcod {
    root: Root,
    con: Offscreen,
    fov: FovMap,
}

type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
}

/// A tile of the map and its properties
#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
    explored: bool,
    color: Color,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
            explored: false,
            color: COLOR_PLAINS,
        }
    }

    pub fn mountain() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
            explored: false,
            color: COLOR_MOUNTAIN,
        }
    }

    pub fn hill() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
            explored: false,
            color: COLOR_HILL,
        }
    }

    pub fn forest() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
            explored: false,
            color: COLOR_FOREST,
        }
    }

    pub fn water() -> Self {
        Tile {
            blocked: true,
            block_sight: false,
            explored: false,
            color: COLOR_SEA,
        }
    }
}

/// This is a generic object: the player, a monster, an item, the stairs...
/// It's always represented by a character on screen.
#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object { x, y, char, color }
    }

    /// move by the given amount, if the destination is not blocked
    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
        let next_x = if self.x + dx >= MAP_WIDTH {
            0
        } else if self.x + dx < 0 {
            MAP_WIDTH - 1
        } else {
            self.x + dx
        };
        let next_y = if self.y + dy >= MAP_HEIGHT {
            0
        } else if self.y + dy < 0 {
            MAP_HEIGHT - 1
        } else {
            self.y + dy
        };
        if !game.map[next_x as usize][next_y as usize].blocked {
            self.x = next_x;
            self.y = next_y;
        }
    }

    /// set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}

fn make_map() -> Map {
    // fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let perlin = Perlin::new();
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let mut height = 0.0;
            let fertility = perlin.get([x as f64 / 10f64, y as f64 / 10f64, 1.999_282_82]);
            let mountain_modifier = perlin.get([x as f64 / 10f64, y as f64 / 10f64, 1.35424]);
            height += perlin.get([x as f64 / 10f64, y as f64 / 10f64, GAME_SEED]);
            height += perlin.get([x as f64, y as f64, GAME_SEED + 1.0]) / 7.5;

            if height >= -0.1 {
                height += mountain_modifier.abs()
            }

            if height >= 1.1 {
                map[x as usize][y as usize] = Tile::mountain()
            } else if height >= 0.50 {
                map[x as usize][y as usize] = Tile::hill()
            } else if height < -0.175 {
                map[x as usize][y as usize] = Tile::water()
            } else if fertility >= 0.25 {
                map[x as usize][y as usize] = Tile::forest()
            }
        }
    }
    map
}

fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {
    if fov_recompute {
        // recompute FOV if needed (the player moved or something)
        let player = &objects[0];
        tcod.fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    // go through all tiles, and set their background color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = tcod.fov.is_in_fov(x, y);
            let tile = game.map[x as usize][y as usize];
            let color = if visible {
                let explored = &mut game.map[x as usize][y as usize].explored;
                *explored = true;
                tile.color
            } else if !tile.explored {
                Color {
                    r: 242,
                    g: 227,
                    b: 211,
                }
            } else {
                Color {
                    r: tile.color.r / 3,
                    g: tile.color.g / 3,
                    b: tile.color.b / 3,
                }
            };
            tcod.con
                .set_char_background(x, y, color, BackgroundFlag::Set);
        }
    }

    // draw all objects in the list
    for object in objects {
        object.draw(&mut tcod.con);
    }

    // blit the contents of "con" to the root console
    blit(
        &tcod.con,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
}

fn handle_keys(tcod: &mut Tcod, game: &Game, player: &mut Object) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = tcod.root.wait_for_keypress(true);
    match key {
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        }
        Key { code: Escape, .. } => return true, // exit game

        // movement keys
        Key { code: Up, .. } => player.move_by(0, -1, game),
        Key { code: Down, .. } => player.move_by(0, 1, game),
        Key { code: Left, .. } => player.move_by(-1, 0, game),
        Key { code: Right, .. } => player.move_by(1, 0, game),

        _ => {}
    }

    false
}

fn main() {
    tcod::system::set_fps(LIMIT_FPS);

    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust/libtcod tutorial")
        .init();

    let mut tcod = Tcod {
        root,
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
    };

    // create object representing the player
    let player = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', WHITE);

    // create an NPC
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', YELLOW);

    // the list of objects with those two
    let mut objects = [player, npc];

    let mut game = Game {
        // generate map (at this point it's not drawn to the screen)
        map: make_map(),
    };

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                !game.map[x as usize][y as usize].block_sight,
                !game.map[x as usize][y as usize].blocked,
            );
        }
    }

    let mut previous_player_position = (-1, -1);

    while !tcod.root.window_closed() {
        // clear the screen of the previous frame
        tcod.con.clear();

        // render the screen
        let fov_recompute = previous_player_position != (objects[0].x, objects[0].y);
        render_all(&mut tcod, &mut game, &objects, fov_recompute);

        tcod.root.flush();

        // handle keys and exit game if needed
        let player = &mut objects[0];
        previous_player_position = (player.x, player.y);
        let exit = handle_keys(&mut tcod, &game, player);
        if exit {
            break;
        }
    }
}
