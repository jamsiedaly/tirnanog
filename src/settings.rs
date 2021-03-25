pub mod settings {
    use tcod::Color;
    use tcod::map::FovAlgorithm;

    pub const MAP_WIDTH: i32 = 1000;
    pub const MAP_HEIGHT: i32 = 450;
    pub const GAME_SEED: f64 = 1.5;

    pub const FOV_ALGO: FovAlgorithm = FovAlgorithm::Shadow; // default FOV algorithm
    pub const FOV_LIGHT_WALLS: bool = true; // light walls or not
    pub const TORCH_RADIUS: i32 = 15;

    pub const LIMIT_FPS: i32 = 60; // 60 frames-per-second maximum

    pub const BAR_WIDTH: i32 = 20;
    pub const PANEL_HEIGHT: i32 = 7;
    // pub const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;

    pub const COLOR_MOUNTAIN: Color = Color {
        r: 244,
        g: 251,
        b: 252,
    };
    pub const COLOR_HILL: Color = Color {
        r: 214,
        g: 163,
        b: 110,
    };
    pub const COLOR_SEA: Color = Color {
        r: 127,
        g: 191,
        b: 191,
    };
    pub const COLOR_FOREST: Color = Color {
        r: 127,
        g: 191,
        b: 127,
    };
    pub const COLOR_PLAINS: Color = Color {
        r: 161,
        g: 214,
        b: 110,
    };
    pub const COLOR_VILLAGE: Color = Color {
        r: 161,
        g: 144,
        b: 110,
    };
}