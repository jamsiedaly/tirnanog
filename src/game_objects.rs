pub mod game_objects {
    use crate::settings::settings::{COLOR_PLAINS, COLOR_MOUNTAIN, COLOR_HILL, COLOR_FOREST, COLOR_SEA};
    use tcod::{Color, Console, BackgroundFlag};
    use tcod::console::{Root, Offscreen};
    use tcod::map::{ Map as FovMap};
    use legion::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(crate) struct Position {
        pub x: i32,
        pub y: i32
    }

    impl Position {
        pub fn new(x: i32, y: i32) -> Position {
            return Position { x, y };
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(crate) struct Drawable {
        char: char,
        color: Color
    }

    impl Drawable {
        pub fn new(char: char, color: Color) -> Drawable {
            return Drawable { char, color };
        }
        pub fn draw(&self, canvas: &mut dyn Console, x: i32, y: i32) {
            canvas.set_default_foreground(self.color);
            canvas.put_char(x, y, self.char, BackgroundFlag::None);
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(crate) struct Vision {
        pub grants_vision: bool
    }

    impl Vision {
        pub fn new(active: bool) -> Vision {
            return Vision { grants_vision: active }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(crate) struct Player {
        pub alive: bool
    }

    impl Player {
        pub fn new(alive: bool) -> Player {
            return Player { alive }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(crate) struct House {
        pub time_since_last_spawn: u128,
        pub population: i32
    }

    impl House {
        pub const TIME_BETWEEN_SPAWNS: u128 = 5000;

        pub fn new() -> House {
            return House {
                population: 0,
                time_since_last_spawn: 0
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub(crate) struct Person {
        name: String,
        pub home: Position,
        pub time_since_last_action: u128
    }

    impl Person {
        pub const TIME_BETWEEN_ACTIONS: u128 = 1000;

        pub fn new(x: i32, y: i32) -> Person {
            return Person {
                name: String::from("Bob"),
                home: Position::new(x, y),
                time_since_last_action: 0
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Tile {
        blocked: bool,
        block_sight: bool,
        pub(crate) explored: bool,
        buildable: bool,
        pub(crate) color: Color,
    }

    pub struct Tcod {
        pub root: Root,
        pub con: Offscreen,
        pub panel: Offscreen,
        pub fov: FovMap
    }

    pub struct GameMap {
        tiles: Vec<Vec<Tile>>
    }

    impl GameMap {
        pub fn new(tiles: Vec<Vec<Tile>>) -> GameMap {
            return GameMap { tiles };
        }

        pub fn is_tile_blocked(&self, x: i32, y: i32) -> bool {
            return self.tiles[x as usize][y as usize].blocked;
        }

        pub fn is_buildable(&self, x: i32, y: i32) -> bool {
            return self.tiles[x as usize][y as usize].buildable;
        }


        pub fn make_tile_built_on(&mut self, x: i32, y: i32) {
            self.tiles[x as usize][y as usize].blocked = true;
            self.tiles[x as usize][y as usize].buildable = false;
        }

        pub fn is_tile_blocking_vision(&self, x: usize, y: usize) -> bool {
            return self.tiles[x][y].block_sight;
        }

        pub fn get_tile(&self, x: usize, y: usize) -> Tile {
            return self.tiles[x][y];
        }

        pub fn set_tile_explored(&mut self, explored: bool, x: usize, y: usize) {
            self.tiles[x][y].explored = explored;
        }
    }

    pub struct Game {
        pub(crate) map: GameMap,
        pub(crate) camera_height: i32,
        pub(crate) camera_width: i32,
        pub(crate) population: i32,
        pub(crate) wood: i32,
        pub(crate) food: i32,
        pub(crate) world: World
    }

    impl Tile {
        pub fn is_blocked(&self,) -> bool {
            return self.blocked;
        }

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

    #[derive(PartialEq, Debug)]
    pub(crate) enum Action {
        MoveUp,
        MoveDown,
        MoveLeft,
        MoveRight,
        Build,
        FullScreen,
        Quit,
    }

}