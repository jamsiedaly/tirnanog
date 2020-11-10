use noise::{NoiseFn, Perlin};
use tcod::colors::*;
use tcod::console::*;
use tcod::map::{FovAlgorithm, Map as FovMap};
use rand::Rng;

const CAMERA_WIDTH: i32 = 40;
const CAMERA_HEIGHT: i32 = 30;


// size of the map
const MAP_WIDTH: i32 = 1000;
const MAP_HEIGHT: i32 = 450;
const GAME_SEED: f64 = 1.5;

// actual size of the window
const SCREEN_WIDTH: i32 = CAMERA_WIDTH;
const SCREEN_HEIGHT: i32 = CAMERA_HEIGHT;

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
const COLOR_VILLAGE: Color = Color {
    r: 161,
    g: 144,
    b: 110,
};
// const COLOR_TOWN: Color = Color { r: 161, g: 140, b: 118 };

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Shadow; // default FOV algorithm
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
    buildable: bool,
    color: Color,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
            explored: false,
            buildable: true,
            color: COLOR_PLAINS,
        }
    }

    pub fn mountain() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
            explored: false,
            buildable: false,
            color: COLOR_MOUNTAIN,
        }
    }

    pub fn hill() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
            explored: false,
            buildable: false,
            color: COLOR_HILL,
        }
    }

    pub fn forest() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
            explored: false,
            buildable: true,
            color: COLOR_FOREST,
        }
    }

    pub fn water() -> Self {
        Tile {
            blocked: true,
            block_sight: false,
            explored: false,
            buildable: false,
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
    grants_vision: bool,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color, grants_vision: bool) -> Self {
        Object { x, y, char, color, grants_vision }
    }

    /// set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
}

fn build_village(x: i32, y: i32, game: &Game, objects: &mut Vec<Object>) {
    let current_tile = game.map[x as usize ][y as usize];
    if current_tile.buildable {
        let village = Object::new(x, y, '1', COLOR_VILLAGE, true);
        objects.push(village);
    }
}

fn move_by(id: usize, dx: i32, dy: i32, game: &Game, objects: &mut Vec<Object>) {
    let (x, y) = objects[id].pos();
    let x = x - MAP_WIDTH;
    let y = y - MAP_HEIGHT;
    let next_x = if x + dx >= MAP_WIDTH {
        0
    } else if x + dx < 0 {
        MAP_WIDTH - 1
    } else {
        x + dx
    };
    let next_y = if y + dy >= MAP_HEIGHT {
        0
    } else if y + dy < 0 {
        MAP_HEIGHT - 1
    } else {
        y + dy
    };

    if !game.map[next_x as usize][next_y as usize].blocked {
        objects[id].set_pos(next_x + MAP_WIDTH, next_y + MAP_HEIGHT);
    }
}

fn make_map() -> Map {
    // fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::empty(); (MAP_HEIGHT*3) as usize]; (MAP_WIDTH*3) as usize];
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

            for i in 0..3 {
                for j in 0..3 {
                    if height >= 1.1 {
                        map[(x + (MAP_WIDTH*i))  as usize][(y + (MAP_HEIGHT*j)) as usize] = Tile::mountain()
                    } else if height >= 0.50 {
                        map[(x + (MAP_WIDTH*i)) as usize][(y + (MAP_HEIGHT*j)) as usize] = Tile::hill()
                    } else if height < -0.175 {
                        map[(x + (MAP_WIDTH*i)) as usize][(y + (MAP_HEIGHT*j)) as usize] = Tile::water()
                    } else if fertility >= 0.25 {
                        map[(x + (MAP_WIDTH*i)) as usize][(y + (MAP_HEIGHT*j)) as usize] = Tile::forest()
                    }
                }
            }


        }
    }
    map
}

fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {
    let player = &objects[0];
    if fov_recompute {
        // recompute FOV if needed (the player moved or something)
        tcod.fov
            .compute_fov(
            player.x,
            player.y,
            TORCH_RADIUS,
            FOV_LIGHT_WALLS,
            FOV_ALGO
        );
    }

    let top = player.y - (CAMERA_HEIGHT / 2);
    let bottom = player.y + (CAMERA_HEIGHT / 2);
    let left = player.x - (CAMERA_WIDTH / 2);
    let right = player.x + (CAMERA_WIDTH / 2);

    for y in top..bottom {
        for x in left..right {
            for vertical_offset in -1..2 {
                for horizontal_offset in -1..2 {
                    let visible = tcod.fov.is_in_fov(x, y);
                    let mut x = x + (MAP_WIDTH * horizontal_offset);
                    x = if x < 0 { 0 } else if x >= (MAP_WIDTH*3) { (MAP_WIDTH*3) - 1 } else {x};
                    let mut y = y + (MAP_HEIGHT * vertical_offset);
                    y = if y < 0 { 0 } else if y >= (MAP_HEIGHT*3) { (MAP_HEIGHT*3) - 1 } else {y};
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
                    if vertical_offset == 0 && horizontal_offset == 0 {
                        tcod.con
                            .set_char_background(x, y, color, BackgroundFlag::Set);
                    }
                }
            }
        }
    }

    // draw all objects in the list
    for object in objects {
        object.draw(&mut tcod.con);
    }

    let source_x = (player.x - (CAMERA_WIDTH / 2));
    let source_y = (player.y - (CAMERA_HEIGHT / 2));

    // blit the contents of "con" to the root console
    blit(
        &tcod.con,
        (source_x, source_y),
        (CAMERA_WIDTH, CAMERA_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
    // blit(
    //     &tcod.con,
    //     (0, 0),
    //     (MAP_WIDTH*3, MAP_HEIGHT*3),
    //     &mut tcod.root,
    //     (0, 0),
    //     1.0,
    //     1.0,
    // );
}

fn handle_keys(tcod: &mut Tcod, game: &Game, objects: &mut Vec<Object>) -> bool {
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
        Key { code: Up, .. } => move_by(0, 0, -1, game, objects),
        Key { code: Down, .. } => move_by(0, 0, 1, game, objects),
        Key { code: Left, .. } => move_by(0,-1, 0, game, objects),
        Key { code: Right, .. } => move_by(0, 1, 0, game, objects),
        Key { code: Spacebar, .. } => build_village(objects[0].x, objects[0].y, game, objects),

        _ => {}
    }

    false
}

fn surrounded_by_land(x: i32, y: i32, map: &Map) -> bool {
    for x_offset in -1..1 {
        for y_offset in -1..1 {
            let tile: Tile = map[(x+x_offset).abs() as usize][(y+y_offset).abs() as usize];
            if tile.blocked {
                return false;
            }
        }
    }
    return true;
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
        con: Offscreen::new(MAP_WIDTH*3, MAP_HEIGHT*3),
        fov: FovMap::new(MAP_WIDTH*3, MAP_HEIGHT*3),
    };

    let mut game = Game {
        // generate map (at this point it's not drawn to the screen)
        map: make_map(),
    };

    let player = loop {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0, MAP_WIDTH);
        let y = rng.gen_range(0, MAP_HEIGHT);
        if surrounded_by_land(x, y, &game.map) {
            break Object::new(x + MAP_WIDTH, y + MAP_HEIGHT, '@', WHITE, true);
        }
    };

    let mut objects = vec![player];

    for y in 0..MAP_HEIGHT*3 {
        for x in 0..MAP_WIDTH*3 {
            tcod.fov.set(
                x ,
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
        let exit = handle_keys(&mut tcod, &mut game, &mut objects);
        if exit {
            break;
        }
    }
}
