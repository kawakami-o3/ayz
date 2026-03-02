use std::collections::VecDeque;
use std::io::{Stdout, Write};

use crossterm::{
    cursor::MoveTo,
    queue,
    terminal::{Clear, ClearType},
};

use crate::core::dungeon::GameState;
use crate::core::types::Position;
use crate::ui::renderer::Renderer;

pub struct TerminalRenderer {
    stdout: Stdout,
    message_log: VecDeque<String>,
    max_messages: usize,
}

impl TerminalRenderer {
    pub fn new(stdout: Stdout) -> Self {
        TerminalRenderer {
            stdout,
            message_log: VecDeque::new(),
            max_messages: 2,
        }
    }

    fn render_header(&mut self, state: &GameState) -> Result<(), std::io::Error> {
        queue!(self.stdout, MoveTo(0, 0))?;
        write!(
            self.stdout,
            "Floor {}/{}",
            state.floor, state.max_floor
        )?;
        // Clear rest of line
        write!(self.stdout, "                              ")?;
        Ok(())
    }

    fn render_map(&mut self, state: &GameState) -> Result<(), std::io::Error> {
        let y_offset: u16 = 1;

        for y in 0..state.map.height {
            for x in 0..state.map.width {
                let pos = Position::new(x as i32, y as i32);
                let c = state.map.render_char(&pos);
                queue!(self.stdout, MoveTo(x as u16, y_offset + y as u16))?;
                write!(self.stdout, "{}", c)?;
            }
        }

        // Draw monsters
        for m in &state.monsters {
            queue!(
                self.stdout,
                MoveTo(m.pos.x as u16, y_offset + m.pos.y as u16)
            )?;
            write!(self.stdout, "{}", m.symbol)?;
        }

        // Draw player
        queue!(
            self.stdout,
            MoveTo(
                state.player.pos.x as u16,
                y_offset + state.player.pos.y as u16
            )
        )?;
        write!(self.stdout, "{}", state.player.symbol)?;

        Ok(())
    }

    fn render_status(&mut self, state: &GameState) -> Result<(), std::io::Error> {
        let status_y = state.map.height as u16 + 1;
        queue!(self.stdout, MoveTo(0, status_y))?;

        let next_exp = self.next_level_exp(state);
        write!(
            self.stdout,
            "HP: {}/{}  Lv: {}  Exp: {}/{}  Floor: {}/{}    ",
            state.player.hp,
            state.player.max_hp,
            state.player.level,
            state.player.exp,
            next_exp,
            state.floor,
            state.max_floor,
        )?;

        Ok(())
    }

    fn render_messages(&mut self, state: &GameState) -> Result<(), std::io::Error> {
        let base_y = state.map.height as u16 + 2;

        for (i, msg) in self.message_log.iter().enumerate() {
            queue!(self.stdout, MoveTo(0, base_y + i as u16))?;
            write!(self.stdout, "{:<80}", msg)?;
        }

        // Clear unused message lines
        for i in self.message_log.len()..self.max_messages {
            queue!(self.stdout, MoveTo(0, base_y + i as u16))?;
            write!(self.stdout, "{:<80}", "")?;
        }

        Ok(())
    }

    fn next_level_exp(&self, state: &GameState) -> i32 {
        let table = [0, 30, 70, 120, 200];
        let idx = state.player.level as usize;
        if idx < table.len() {
            table[idx]
        } else {
            999
        }
    }
}

impl Renderer for TerminalRenderer {
    type Error = std::io::Error;

    fn render(&mut self, state: &GameState) -> Result<(), Self::Error> {
        self.render_header(state)?;
        self.render_map(state)?;
        self.render_status(state)?;
        self.render_messages(state)?;
        self.stdout.flush()?;
        Ok(())
    }

    fn push_message(&mut self, msg: &str) {
        self.message_log.push_back(msg.to_string());
        while self.message_log.len() > self.max_messages {
            self.message_log.pop_front();
        }
    }

    fn render_game_over(&mut self, state: &GameState) -> Result<(), Self::Error> {
        queue!(self.stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        writeln!(self.stdout)?;
        writeln!(self.stdout, "        GAME OVER")?;
        writeln!(self.stdout)?;
        writeln!(self.stdout, "   到達フロア: {}/{}", state.floor, state.max_floor)?;
        writeln!(self.stdout, "   レベル: {}", state.player.level)?;
        writeln!(self.stdout)?;
        writeln!(self.stdout, "   Press any key to exit...")?;
        self.stdout.flush()?;
        Ok(())
    }

    fn render_game_clear(&mut self, state: &GameState) -> Result<(), Self::Error> {
        queue!(self.stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        writeln!(self.stdout)?;
        writeln!(self.stdout, "     CONGRATULATIONS!")?;
        writeln!(self.stdout)?;
        writeln!(self.stdout, "   全 {} フロアを踏破！", state.max_floor)?;
        writeln!(self.stdout, "   レベル: {}", state.player.level)?;
        writeln!(self.stdout)?;
        writeln!(self.stdout, "   Press any key to exit...")?;
        self.stdout.flush()?;
        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), Self::Error> {
        queue!(
            self.stdout,
            Clear(ClearType::All),
            MoveTo(0, 0),
            crossterm::cursor::Show
        )?;
        self.stdout.flush()?;
        Ok(())
    }
}
