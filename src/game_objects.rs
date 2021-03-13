pub mod game_objects {
    use crate::settings::settings::{COLOR_PLAINS, COLOR_MOUNTAIN, COLOR_HILL, COLOR_FOREST, COLOR_SEA};
    use tcod::{Color, Console, BackgroundFlag};
    use tcod::console::{Root, Offscreen};
    use tcod::map::{FovAlgorithm, Map as FovMap};

    /// This is a generic object: the player, a monster, an item, the stairs...
    /// It's always represented by a character on screen.
    #[derive(Debug)]
    pub struct Object {
        pub x: i32,
        pub y: i32,
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

    pub struct Tcod {
        pub root: Root,
        pub con: Offscreen,
        pub fov: FovMap,
    }

    pub(crate) type Map = Vec<Vec<Tile>>;

    pub struct Game {
        pub(crate) map: Map,
        pub(crate) camera_height: i32,
        pub(crate) camera_width: i32
    }

    /// A tile of the map and its properties
    #[derive(Clone, Copy, Debug)]
    pub struct Tile {
        blocked: bool,
        block_sight: bool,
        pub(crate) explored: bool,
        buildable: bool,
        pub(crate) color: Color,
    }

    impl Game {
        pub fn is_tile_blocked(&self, x: usize, y: usize) -> bool {
            return self.map[x][y].blocked;
        }

        pub fn is_tile_explored(&self, x: usize, y: usize) -> bool {
            return self.map[x][y].explored;
        }

        pub fn is_tile_blocking_vision(&self, x: usize, y: usize) -> bool {
            return self.map[x][y].block_sight;
        }

        pub fn get_tile(&self, x: usize, y: usize) -> Tile {
            return self.map[x][y];
        }

        pub fn set_tile_explored(&mut self, explored: bool, x: usize, y: usize) {
            self.map[x][y].explored = explored;
        }
    }

    impl Tile {
        pub fn is_buildable(&self,) -> bool {
            return self.buildable;
        }

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

}