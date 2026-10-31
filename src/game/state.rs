#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    MainMenu,
    Playing,
    PauseMenu,
    Inventory,
    Settings { from_main_menu: bool },
    GameOver,
}
