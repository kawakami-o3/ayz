use crate::core::dungeon::GameState;
use crate::core::turn::{GameCommand, GameEvent};
use crate::ui::input::InputHandler;
use crate::ui::renderer::Renderer;

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

            let events = self.state.process_turn(command);

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
                // Wait for any key
                let _ = self.input.next_command();
                break;
            }

            if events.iter().any(|e| matches!(e, GameEvent::GameClear)) {
                self.renderer.render_game_clear(&self.state)?;
                // Wait for any key
                let _ = self.input.next_command();
                break;
            }

            self.renderer.render(&self.state)?;
        }

        self.renderer.cleanup()?;
        Ok(())
    }
}

fn event_to_message(event: &GameEvent) -> Option<String> {
    match event {
        GameEvent::PlayerAttacked { target_name, damage } => {
            Some(format!("{} に {} ダメージを与えた", target_name, damage))
        }
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
        GameEvent::LevelUp { new_level } => {
            Some(format!("レベルが {} に上がった！", new_level))
        }
        GameEvent::FloorAdvance { new_floor } => {
            Some(format!("{}階に降りた", new_floor))
        }
        GameEvent::Message(msg) => Some(msg.clone()),
        _ => None,
    }
}
