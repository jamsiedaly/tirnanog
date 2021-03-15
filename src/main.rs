use noise::{NoiseFn, Perlin};
use tcod::colors::*;
use tcod::console::*;
use tcod::map::{ Map as FovMap};
use rand::Rng;
mod settings;
use crate::settings::settings::*;
mod game_objects;
use crate::game_objects::game_objects::*;

fn build_village(x: i32, y: i32, game: &Game, objects: &mut Vec<Object>) {
    let current_tile = game.get_tile(x as usize, y as usize);
    if current_tile.is_buildable() {
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

    if !game.is_tile_blocked(next_x as usize, next_y as usize) {
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
            let mountain_modifier = perlin.get([x as f64 / 10f64, y as f64 / 10f64, 2.5]);
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

    let top = player.y - (game.camera_height / 2);
    let bottom = player.y + (game.camera_height / 2);
    let left = player.x - (game.camera_width / 2);
    let right = player.x + (game.camera_width / 2);

    for y in top..bottom {
        for x in left..right {
            for vertical_offset in -1..2 {
                for horizontal_offset in -1..2 {
                    let visible = tcod.fov.is_in_fov(x, y);
                    let mut x = x + (MAP_WIDTH * horizontal_offset);
                    x = if x < 0 { 0 } else if x >= (MAP_WIDTH*3) { (MAP_WIDTH*3) - 1 } else {x};
                    let mut y = y + (MAP_HEIGHT * vertical_offset);
                    y = if y < 0 { 0 } else if y >= (MAP_HEIGHT*3) { (MAP_HEIGHT*3) - 1 } else {y};
                    let tile = game.get_tile(x as usize, y as usize);
                    let color = if visible {
                        game.set_tile_explored(true, x as usize, y as usize);
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

    let source_x = player.x - (game.camera_width / 2);
    let source_y = player.y - (game.camera_height / 2);

    blit(
        &tcod.con,
        (source_x, source_y),
        (game.camera_width, game.camera_height),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );

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
            return !tile.is_blocked();
        }
    }
    return true;
}

fn main() {
    tcod::system::set_fps(LIMIT_FPS);
    let (screen_width, screen_height) = tcod::system::get_current_resolution();
    let pixel_width = screen_width / 20;
    let pixel_height = screen_height / 20;


    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(pixel_width, pixel_height)
        .fullscreen(true)
        .title("Rouge Civ")
        .init();

    let mut tcod = Tcod {
        root,
        con: Offscreen::new(MAP_WIDTH*3, MAP_HEIGHT*3),
        fov: FovMap::new(MAP_WIDTH*3, MAP_HEIGHT*3),
    };


    let mut game = Game {
        map: make_map(),
        camera_width: pixel_width,
        camera_height: pixel_height
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
                !game.is_tile_blocking_vision(x as usize,y as usize),
                !game.is_tile_blocked(x as usize,y as usize),
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
