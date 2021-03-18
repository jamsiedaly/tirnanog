use noise::{NoiseFn, Perlin};
use tcod::colors::*;
use tcod::console::*;
use tcod::map::{ Map as FovMap};
use rand::Rng;
mod settings;
use crate::settings::settings::*;
mod game_objects;
use crate::game_objects::game_objects::*;
use legion::{World, IntoQuery, Entity};
use tcod::input::KEY_PRESSED;

fn make_map() -> GameMap {
    let mut tiles = vec![vec![Tile::empty(); (MAP_HEIGHT*3) as usize]; (MAP_WIDTH*3) as usize];
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
                        tiles[(x + (MAP_WIDTH*i))  as usize][(y + (MAP_HEIGHT*j)) as usize] = Tile::mountain()
                    } else if height >= 0.50 {
                        tiles[(x + (MAP_WIDTH*i)) as usize][(y + (MAP_HEIGHT*j)) as usize] = Tile::hill()
                    } else if height < -0.175 {
                        tiles[(x + (MAP_WIDTH*i)) as usize][(y + (MAP_HEIGHT*j)) as usize] = Tile::water()
                    } else if fertility >= 0.25 {
                        tiles[(x + (MAP_WIDTH*i)) as usize][(y + (MAP_HEIGHT*j)) as usize] = Tile::forest()
                    }
                }
            }


        }
    }
    GameMap::new(tiles)
}

fn render_all(tcod: &mut Tcod, game: &mut Game, fov_recompute: bool, player: Entity) {

    if fov_recompute {
        let mut query = <(&Vision, &mut Position)>::query();
        let things_with_vision = query.iter_mut(&mut game.world);

        for (vision, position) in things_with_vision {
            if vision.grants_vision {
                tcod.fov.compute_fov(
                position.x,
                position.y,
                TORCH_RADIUS,
                FOV_LIGHT_WALLS,
                FOV_ALGO
                );
            }
        }
    }

    let mut query = <&Position>::query();
    let position = query.get(&mut game.world, player).unwrap().clone();

    let top = position.y - (game.camera_height / 2);
    let bottom = position.y + (game.camera_height / 2);
    let left = position.x - (game.camera_width / 2);
    let right = position.x + (game.camera_width / 2);

    for y in top..bottom {
        for x in left..right {
            for vertical_offset in -1..2 {
                for horizontal_offset in -1..2 {
                    let visible = tcod.fov.is_in_fov(x, y);
                    let mut x = x + (MAP_WIDTH * horizontal_offset);
                    x = if x < 0 { 0 } else if x >= (MAP_WIDTH*3) { (MAP_WIDTH*3) - 1 } else {x};
                    let mut y = y + (MAP_HEIGHT * vertical_offset);
                    y = if y < 0 { 0 } else if y >= (MAP_HEIGHT*3) { (MAP_HEIGHT*3) - 1 } else {y};
                    if visible {
                        game.map.set_tile_explored(true, x as usize, y as usize);
                    }
                }
            };
            let visible = tcod.fov.is_in_fov(x, y);
            let tile = game.map.get_tile(x as usize, y as usize);
            let color = if visible {
                game.map.set_tile_explored(true, x as usize, y as usize);
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

    let mut query = <(&Drawable, &Position)>::query();
    for (drawable, position) in query.iter_mut(&mut game.world) {
        drawable.draw(&mut tcod.con, position.x, position.y)
    }


    let source_x = position.x - (game.camera_width / 2);
    let source_y = position.y - (game.camera_height / 2);

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

fn handle_keys(tcod: &mut Tcod, game: &mut Game) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key_option = tcod.root.check_for_keypress(KEY_PRESSED);
    match key_option {
        Some(key) => match key {
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
            Key { code: Up, .. } => {
                let mut query = <(&Player, &mut Position)>::query();
                let position = query.iter_mut(&mut game.world).next().unwrap().1;
                position.y -= 1;
                if position.y < MAP_HEIGHT { position.y = MAP_HEIGHT*2 -1 }
                let (x, y) = (position.x, position.y);
                if game.map.is_tile_blocked(x, y) {
                    position.y += 1;
                }
            },
            Key { code: Down, .. } => {
                let mut query = <(&Player, &mut Position)>::query();
                let position = query.iter_mut(&mut game.world).next().unwrap().1;
                position.y += 1;
                if position.y >= MAP_HEIGHT*2 { position.y = 0 + MAP_HEIGHT}
                if game.map.is_tile_blocked(position.x, position.y) {
                    position.y -= 1;
                }
            },
            Key { code: Left, .. } => {
                let mut query = <(&Player, &mut Position)>::query();
                let position = query.iter_mut(&mut game.world).next().unwrap().1;
                position.x -= 1;
                if position.x < MAP_WIDTH { position.x = MAP_WIDTH*2 -1 }
                if game.map.is_tile_blocked(position.x, position.y) {
                    position.x += 1;
                }
            },
            Key { code: Right, .. } => {
                let mut query = <(&Player, &mut Position)>::query();
                let position = query.iter_mut(&mut game.world).next().unwrap().1;
                position.x += 1;
                if position.x >= MAP_WIDTH*2 { position.x = MAP_WIDTH }
                if game.map.is_tile_blocked(position.x, position.y) {
                    position.x -= 1;
                }
            },
            Key { code: Spacebar, .. } => {
                let mut query = <(&Player,&Position)>::query();
                let player = query.iter(&game.world).next().unwrap();
                if game.map.is_buildable(player.1.x, player.1.y) {
                    game.map.make_tile_built_on(player.1.x, player.1.y);
                    game.world.push((
                        Position::new(player.1.x, player.1.y),
                        Drawable::new('A', COLOR_VILLAGE),
                        House::new()
                    ));
                }
            },
            _ => {}
        }
        _ => {}
    }
    false
}

fn surrounded_by_land(x: i32, y: i32, map: &GameMap) -> bool {
    for x_offset in -1..1 {
        for y_offset in -1..1 {
            let tile: Tile = map.get_tile((x+x_offset).abs() as usize,(y+y_offset).abs() as usize);
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
        camera_height: pixel_height,
        world: World::default()
    };

    let player = loop {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0, MAP_WIDTH);
        let y = rng.gen_range(0, MAP_HEIGHT);
        if surrounded_by_land(x, y, &game.map) {
            break game.world.push((
                Position::new(x + MAP_WIDTH, y + MAP_HEIGHT),
                Drawable::new('@', WHITE),
                Vision::new(true),
                Player::new(true)
            ))
        }
    };

    for y in 0..MAP_HEIGHT*3 {
        for x in 0..MAP_WIDTH*3 {
            tcod.fov.set(
                x ,
                y,
                !game.map.is_tile_blocking_vision(x as usize,y as usize),
                !game.map.is_tile_blocked(x,y),
            );
        }
    }

    let previous_player_position = (-1, -1);

    while !tcod.root.window_closed() {
        // clear the screen of the previous frame
        tcod.con.clear();

        let mut player_query = <(&Player,&Position)>::query();
        let players_position = player_query.iter(&game.world).next().unwrap().1;

        let fov_recompute = previous_player_position != (players_position.x, players_position.y);
        render_all(&mut tcod, &mut game, fov_recompute, player);
        tcod.root.flush();

        let exit = handle_keys(&mut tcod, &mut game);
        if exit {
            break;
        }
    }
}
