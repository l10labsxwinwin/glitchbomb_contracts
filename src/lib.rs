pub enum Game {
    New,
    InLevel { game_data: GameData },
    InShop { game_data: GameData },
    Complete { moonrocks_diff: i32 },
}

pub enum Actions {
    StartGame,
    PullOrb,
    CashOut,
    EnterShop,
    BuyOrb,
    GoToNextLevel,
}

#[derive(Default)]
pub struct GameData {
    pub level: u32,
    pub points: u32,
    pub milestone: u32,
    pub hp: u32,
    pub max_hp: u32,
    pub multiplier: f32,
    pub glitch_chips: u32,
    pub moonrocks_spent: u32,
    pub moonrocks_earned: u32,
    pub all_orbs: Vec<Orb>,
    pub pullable_orb_effects: Vec<OrbEffect>,
    pub pulled_orbs_effects: Vec<OrbEffect>,
}

pub struct Orb {
    pub orb_effect: OrbEffect,
    pub rarity: OrbRarity,
    pub count: u32,
    pub base_price: u32,
    pub current_price: u32,
}

pub enum OrbRarity {
    Common,
    Rare,
    Cosmic,
}

pub enum OrbEffect {
    Health { healing: u32 },
    Point { points: u32 },
    Bomb { damage: u32 },
    Multiplier { additional_mult: f32 },
}
