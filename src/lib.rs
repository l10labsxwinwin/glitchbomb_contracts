use rand::seq::IndexedRandom;

pub enum Game {
    New,
    Level { game_data: GameData },
    Shop { game_data: GameData },
    Complete { moonrocks_diff: i32 },
}

pub enum Action {
    StartGame,
    PullOrb,
    CashOut,
    EnterShop,
    BuyOrb,
    GoToNextLevel,
}

pub enum ActionError {
    InvalidActionInNewGame,
    InvalidActionInLevel,
    InvalidActionInShop,
    NoPointsToCashOut,
    MilestoneNotMetYet,
    GameOver,
}

pub fn perform_action(game: &mut Game, action: Action) -> Result<(), ActionError> {
    match (&game, action) {
        (Game::New, Action::StartGame) => {
            *game = Game::Level {
                game_data: GameData::new(),
            };
            Ok(())
        }
        (Game::New, _) => Err(ActionError::InvalidActionInNewGame),
        (Game::Level { game_data }, Action::CashOut) => match game_data.points == 0 {
            true => Err(ActionError::NoPointsToCashOut),
            false => {
                let mut moonrocks_diff = 0;
                moonrocks_diff += game_data.points as i32;
                moonrocks_diff += game_data.moonrocks_earned as i32;
                moonrocks_diff -= game_data.moonrocks_spent as i32;
                *game = Game::Complete { moonrocks_diff };
                Ok(())
            }
        },
        (Game::Level { game_data }, Action::PullOrb) => todo!(),
        (Game::Level { game_data }, Action::EnterShop) => {
            match game_data.points >= game_data.milestone {
                true => {
                    // filter buyable orbs of each rarity
                    let common_indices: Vec<usize> = game_data
                        .all_orbs
                        .iter()
                        .enumerate()
                        .filter(|(_, orb)| orb.is_common() && orb.is_buyable())
                        .map(|(i, _)| i)
                        .collect();
                    let rare_indices: Vec<usize> = game_data
                        .all_orbs
                        .iter()
                        .enumerate()
                        .filter(|(_, orb)| orb.is_rare() && orb.is_buyable())
                        .map(|(i, _)| i)
                        .collect();
                    let cosmic_indices: Vec<usize> = game_data
                        .all_orbs
                        .iter()
                        .enumerate()
                        .filter(|(_, orb)| orb.is_cosmic() && orb.is_buyable())
                        .map(|(i, _)| i)
                        .collect();

                    // randomly select 3 common, 2 rare, and 1 cosmic for shop
                    let mut rng = rand::rng();
                    let mut sale_orbs_indices = Vec::new();

                    let selected_common: Vec<usize> = common_indices
                        .choose_multiple(&mut rng, 3)
                        .cloned()
                        .collect();

                    let selected_rare: Vec<usize> =
                        rare_indices.choose_multiple(&mut rng, 2).cloned().collect();

                    let selected_cosmic: Vec<usize> = cosmic_indices
                        .choose_multiple(&mut rng, 1)
                        .cloned()
                        .collect();

                    sale_orbs_indices.extend(selected_common);
                    sale_orbs_indices.extend(selected_rare);
                    sale_orbs_indices.extend(selected_cosmic);

                    let game_data = GameData {
                        sale_orbs_indices,
                        ..game_data.clone()
                    };

                    *game = Game::Shop { game_data };
                    Ok(())
                }
                false => Err(ActionError::MilestoneNotMetYet),
            }
        }
        (Game::Level { .. }, _) => Err(ActionError::InvalidActionInLevel),
        (Game::Shop { game_data }, Action::BuyOrb) => todo!(),
        (Game::Shop { game_data }, Action::GoToNextLevel) => {
            *game = Game::Level {
                game_data: GameData::next_level_game_data(game_data),
            };
            Ok(())
        }
        (Game::Shop { .. }, _) => Err(ActionError::InvalidActionInShop),
        (Game::Complete { .. }, _) => Err(ActionError::GameOver),
    }
}

#[derive(Clone)]
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
    pub all_orbs: [Orb; 21],
    pub sale_orbs_indices: Vec<usize>,
    pub pullable_orb_effects: Vec<OrbEffect>,
    pub pulled_orbs_effects: Vec<OrbEffect>,
}

impl GameData {
    const MILESTONES: [u32; 7] = [12, 18, 28, 44, 70, 100, 150];

    pub fn new() -> Self {
        let all_orbs = Orb::all_orbs();
        let pullable_orb_effects = all_orbs
            .iter()
            .flat_map(|orb| orb.to_orb_effects())
            .collect();

        Self {
            level: 1,
            points: 0,
            milestone: Self::MILESTONES[0],
            hp: 5,
            max_hp: 5,
            multiplier: 1.0,
            glitch_chips: 0,
            moonrocks_spent: 10,
            moonrocks_earned: 0,
            all_orbs,
            sale_orbs_indices: Vec::new(),
            pullable_orb_effects,
            pulled_orbs_effects: Vec::new(),
        }
    }

    pub fn next_level_game_data(&self) -> Self {
        let new_game_data = GameData::new();
        let pullable_orb_effects: Vec<OrbEffect> = self
            .all_orbs
            .iter()
            .flat_map(|orb| orb.to_orb_effects())
            .collect();

        GameData {
            level: self.level + 1,
            milestone: Self::MILESTONES[self.level as usize],
            glitch_chips: self.glitch_chips,
            moonrocks_spent: self.moonrocks_spent,
            moonrocks_earned: self.moonrocks_earned,
            all_orbs: self.all_orbs,
            pullable_orb_effects,
            ..new_game_data
        }
    }
}

#[derive(Clone, Copy)]
pub struct Orb {
    pub effect: OrbEffect,
    pub rarity: OrbRarity,
    pub count: u32,
    pub buyable: Buyable,
}

impl Orb {
    pub fn new(effect: OrbEffect, rarity: OrbRarity, count: u32, buyable: Buyable) -> Self {
        Orb {
            effect,
            rarity,
            count,
            buyable,
        }
    }

    pub fn bomb(damage: u32, count: u32, buyable: Buyable) -> Self {
        Self::new(OrbEffect::Bomb(damage), OrbRarity::Common, count, buyable)
    }

    pub fn point(points: u32, count: u32, rarity: OrbRarity, buyable: Buyable) -> Self {
        Self::new(OrbEffect::Point(points), rarity, count, buyable)
    }

    pub fn point_per_orb_remaining(
        points_per_orb: u32,
        count: u32,
        rarity: OrbRarity,
        buyable: Buyable,
    ) -> Self {
        Self::new(
            OrbEffect::PointPerOrbRemaining(points_per_orb),
            rarity,
            count,
            buyable,
        )
    }

    pub fn point_per_bomb_pulled(
        points_per_bomb: u32,
        count: u32,
        rarity: OrbRarity,
        buyable: Buyable,
    ) -> Self {
        Self::new(
            OrbEffect::PointPerBombPulled(points_per_bomb),
            rarity,
            count,
            buyable,
        )
    }

    pub fn glitch_chips(chips: u32, count: u32, rarity: OrbRarity, buyable: Buyable) -> Self {
        Self::new(OrbEffect::GlitchChips(chips), rarity, count, buyable)
    }

    pub fn moonrocks(amount: u32, count: u32, rarity: OrbRarity, buyable: Buyable) -> Self {
        Self::new(OrbEffect::Moonrocks(amount), rarity, count, buyable)
    }

    pub fn health(hp: u32, count: u32, rarity: OrbRarity, buyable: Buyable) -> Self {
        Self::new(OrbEffect::Health(hp), rarity, count, buyable)
    }

    pub fn multiplier(mult: f32, count: u32, rarity: OrbRarity, buyable: Buyable) -> Self {
        Self::new(OrbEffect::Multiplier(mult), rarity, count, buyable)
    }

    pub fn point_rewind(count: u32, rarity: OrbRarity, buyable: Buyable) -> Self {
        Self::new(OrbEffect::PointRewind, rarity, count, buyable)
    }

    pub fn five_or_die(count: u32, rarity: OrbRarity, buyable: Buyable) -> Self {
        Self::new(OrbEffect::FiveOrDie, rarity, count, buyable)
    }

    pub fn bomb_immunity(count: u32, rarity: OrbRarity, buyable: Buyable) -> Self {
        Self::new(OrbEffect::BombImmunity, rarity, count, buyable)
    }

    pub fn all_orbs() -> [Orb; 21] {
        [
            // non-buyables
            Self::bomb(1, 2, Buyable::not_buyable()),
            Self::bomb(2, 1, Buyable::not_buyable()),
            Self::bomb(3, 1, Buyable::not_buyable()),
            Self::point_per_orb_remaining(1, 1, OrbRarity::Common, Buyable::not_buyable()),
            // common buyables
            Self::point(5, 3, OrbRarity::Common, Buyable::buyable(5)),
            Self::glitch_chips(15, 0, OrbRarity::Common, Buyable::buyable(5)),
            Self::five_or_die(0, OrbRarity::Common, Buyable::buyable(5)),
            Self::point_per_bomb_pulled(4, 1, OrbRarity::Common, Buyable::buyable(6)),
            Self::point(7, 0, OrbRarity::Common, Buyable::buyable(8)),
            Self::moonrocks(15, 0, OrbRarity::Common, Buyable::buyable(8)),
            Self::point_rewind(0, OrbRarity::Common, Buyable::buyable(8)),
            Self::multiplier(0.5, 0, OrbRarity::Common, Buyable::buyable(9)),
            Self::health(1, 1, OrbRarity::Common, Buyable::buyable(9)),
            // rare buyables
            Self::point(8, 0, OrbRarity::Rare, Buyable::buyable(11)),
            Self::point(9, 0, OrbRarity::Rare, Buyable::buyable(13)),
            Self::multiplier(1.0, 1, OrbRarity::Rare, Buyable::buyable(14)),
            Self::point_per_orb_remaining(2, 0, OrbRarity::Rare, Buyable::buyable(15)),
            Self::multiplier(1.5, 0, OrbRarity::Rare, Buyable::buyable(16)),
            // cosmic buyables
            Self::health(3, 0, OrbRarity::Cosmic, Buyable::buyable(21)),
            Self::moonrocks(40, 0, OrbRarity::Cosmic, Buyable::buyable(23)),
            Self::bomb_immunity(0, OrbRarity::Cosmic, Buyable::buyable(24)),
        ]
    }

    pub fn to_orb_effects(&self) -> Vec<OrbEffect> {
        vec![self.effect.clone(); self.count as usize]
    }

    pub fn is_common(&self) -> bool {
        self.rarity == OrbRarity::Common
    }

    pub fn is_rare(&self) -> bool {
        self.rarity == OrbRarity::Rare
    }

    pub fn is_cosmic(&self) -> bool {
        self.rarity == OrbRarity::Cosmic
    }

    pub fn is_buyable(&self) -> bool {
        match self.buyable {
            Buyable::Yes { .. } => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum OrbRarity {
    Common,
    Rare,
    Cosmic,
}

#[derive(Clone, Copy)]
pub enum OrbEffect {
    Point(u32),
    PointPerOrbRemaining(u32),
    PointPerBombPulled(u32),
    GlitchChips(u32),
    Moonrocks(u32),
    Health(u32),
    Bomb(u32),
    Multiplier(f32),
    PointRewind,
    FiveOrDie,
    BombImmunity,
}

#[derive(Clone, Copy)]
pub enum Buyable {
    No,
    Yes { base_price: u32, current_price: u32 },
}

impl Buyable {
    pub fn not_buyable() -> Self {
        Buyable::No
    }

    pub fn buyable(base_price: u32) -> Self {
        Buyable::Yes {
            base_price,
            current_price: base_price,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pullable_orb_effects_count() {
        let game_data = GameData::new();
        assert_eq!(game_data.pullable_orb_effects.len(), 11);
    }
}
