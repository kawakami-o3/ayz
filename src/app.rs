use crate::core::dungeon::GameState;
use crate::core::entity::ItemCategory;
use crate::core::turn::{GameCommand, GameEvent};
use crate::ui::input::InputHandler;
use crate::ui::renderer::{InventoryAction, Renderer};

pub struct App<R: Renderer, I: InputHandler> {
    state: GameState,
    renderer: R,
    input: I,
}

impl<R, I> App<R, I>
where
    R: Renderer<Error = std::io::Error>,
    I: InputHandler<Error = std::io::Error>,
{
    pub fn new(state: GameState, renderer: R, input: I) -> Self {
        App {
            state,
            renderer,
            input,
        }
    }

    pub fn run(&mut self) -> Result<(), std::io::Error> {
        self.renderer.render(&self.state)?;

        loop {
            let command = self.input.next_command()?;

            if matches!(command, GameCommand::Quit) {
                break;
            }

            let events = self
                .state
                .process_turn(command)
                .expect("invalid game command");

            // Handle inventory request
            if events
                .iter()
                .any(|e| matches!(e, GameEvent::RequestInventory))
            {
                self.handle_inventory()?;
                self.renderer.render(&self.state)?;
                continue;
            }

            // Handle throw inventory request
            if events
                .iter()
                .any(|e| matches!(e, GameEvent::RequestThrowInventory))
            {
                self.handle_throw_inventory()?;
                self.renderer.render(&self.state)?;
                continue;
            }

            // Convert events to messages
            for event in &events {
                if let Some(msg) = event_to_message(event) {
                    self.renderer.push_message(&msg);
                }
            }

            // Check end conditions
            if events.iter().any(|e| matches!(e, GameEvent::GameOver)) {
                self.renderer.render(&self.state)?;
                self.renderer.render_game_over(&self.state)?;
                let _ = self.input.next_command();
                break;
            }

            if events.iter().any(|e| matches!(e, GameEvent::GameClear)) {
                self.renderer.render_game_clear(&self.state)?;
                let _ = self.input.next_command();
                break;
            }

            self.renderer.render(&self.state)?;
        }

        self.renderer.cleanup()?;
        Ok(())
    }

    fn handle_inventory(&mut self) -> Result<(), std::io::Error> {
        let weapon_name = self.state.player.weapon.as_ref().map(|w| w.display_name());
        let shield_name = self.state.player.shield.as_ref().map(|s| s.display_name());

        let action = self.renderer.render_inventory(
            &self.state.player.inventory,
            weapon_name.as_deref(),
            shield_name.as_deref(),
        )?;

        if let Some(InventoryAction::Use(idx)) = action {
            if idx < self.state.player.inventory.len() {
                let category = self.state.player.inventory[idx].category.clone();
                let cmd = match category {
                    ItemCategory::Weapon => GameCommand::EquipWeapon(idx),
                    ItemCategory::Shield => GameCommand::EquipShield(idx),
                    ItemCategory::Staff(_) => GameCommand::UseStaff(idx),
                    _ => GameCommand::UseItem(idx),
                };

                let events = self
                    .state
                    .process_turn(cmd)
                    .expect("invalid inventory index");
                for event in &events {
                    if let Some(msg) = event_to_message(event) {
                        self.renderer.push_message(&msg);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_throw_inventory(&mut self) -> Result<(), std::io::Error> {
        let weapon_name = self.state.player.weapon.as_ref().map(|w| w.display_name());
        let shield_name = self.state.player.shield.as_ref().map(|s| s.display_name());

        let action = self.renderer.render_throw_inventory(
            &self.state.player.inventory,
            weapon_name.as_deref(),
            shield_name.as_deref(),
        )?;

        if let Some(InventoryAction::Throw(idx)) = action {
            if idx < self.state.player.inventory.len() {
                let events = self
                    .state
                    .process_turn(GameCommand::ThrowItem(idx))
                    .expect("invalid inventory index");
                for event in &events {
                    if let Some(msg) = event_to_message(event) {
                        self.renderer.push_message(&msg);
                    }
                }
            }
        }

        Ok(())
    }
}

fn event_to_message(event: &GameEvent) -> Option<String> {
    match event {
        GameEvent::PlayerAttacked {
            target_name,
            damage,
        } => Some(format!("{} に {} ダメージを与えた", target_name, damage)),
        GameEvent::MonsterDefeated { name, exp } => {
            Some(format!("{} を倒した！ 経験値 {} 獲得", name, exp))
        }
        GameEvent::MonsterAttacked { name, damage } => {
            Some(format!("{} の攻撃！ {} ダメージを受けた", name, damage))
        }
        GameEvent::PlayerDamaged { remaining_hp, .. } => {
            if *remaining_hp <= 5 {
                Some(String::from("危険！ HPが残りわずか！"))
            } else {
                None
            }
        }
        GameEvent::ItemPickedUp { name } => Some(format!("{} を拾った", name)),
        GameEvent::ItemUsed { name, effect_desc } => {
            Some(format!("{} を使った。{}", name, effect_desc))
        }
        GameEvent::EquipmentPickedUp { name } => Some(format!("{} を拾った", name)),
        GameEvent::Equipped { name } => Some(format!("{} を装備した", name)),
        GameEvent::InventoryFull => Some(String::from("持ち物がいっぱいだ")),
        GameEvent::Starving => Some(String::from("お腹が空いて力が出ない... HPが減少している")),
        GameEvent::LevelUp { new_level } => Some(format!("レベルが {} に上がった！", new_level)),
        GameEvent::FloorAdvance { new_floor } => Some(format!("{}階に降りた", new_floor)),
        GameEvent::ItemThrown { name, result_desc } => {
            Some(format!("{}を投げた。{}", name, result_desc))
        }
        GameEvent::PlayerHypnotized {
            monster_name,
            action_desc,
        } => Some(format!("{}に催眠をかけられた！{}", monster_name, action_desc)),
        GameEvent::Message(msg) => Some(msg.clone()),
        _ => None,
    }
}
