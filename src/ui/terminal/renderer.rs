use std::collections::VecDeque;
use std::io::{Stdout, Write};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEventKind},
    queue,
    terminal::{Clear, ClearType},
};

use crate::core::dungeon::GameState;
use crate::core::entity::{Item, ItemCategory};
use crate::core::types::Position;
use crate::ui::renderer::{InventoryAction, Renderer};

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
        write!(self.stdout, "                              ")?;
        Ok(())
    }

    fn render_map(&mut self, state: &GameState) -> Result<(), std::io::Error> {
        let y_offset: u16 = 1;
        let vis = &state.visibility;

        for y in 0..state.map.height {
            for x in 0..state.map.width {
                let pos = Position::new(x as i32, y as i32);
                queue!(self.stdout, MoveTo(x as u16, y_offset + y as u16))?;
                if vis.is_visible(&pos) {
                    let c = state.map.render_char(&pos);
                    write!(self.stdout, "{}", c)?;
                } else if vis.is_visited(&pos) {
                    // Dim rendering for visited but not currently visible
                    let c = state.map.render_char(&pos);
                    // Use darker representation
                    let dim_c = match c {
                        '.' => ':',
                        '-' => ';',
                        '+' => '+',
                        _ => ' ',
                    };
                    write!(self.stdout, "{}", dim_c)?;
                } else {
                    write!(self.stdout, " ")?;
                }
            }
        }

        // Draw floor items (only if visible)
        for fi in &state.floor_items {
            if vis.is_visible(&fi.pos) {
                queue!(
                    self.stdout,
                    MoveTo(fi.pos.x as u16, y_offset + fi.pos.y as u16)
                )?;
                write!(self.stdout, "{}", fi.item.symbol)?;
            }
        }

        // Draw floor equipment (only if visible)
        for fe in &state.floor_equips {
            if vis.is_visible(&fe.pos) {
                queue!(
                    self.stdout,
                    MoveTo(fe.pos.x as u16, y_offset + fe.pos.y as u16)
                )?;
                write!(self.stdout, "{}", fe.equip.symbol)?;
            }
        }

        // Draw monsters (only if visible)
        for m in &state.monsters {
            if vis.is_visible(&m.pos) {
                queue!(
                    self.stdout,
                    MoveTo(m.pos.x as u16, y_offset + m.pos.y as u16)
                )?;
                write!(self.stdout, "{}", m.symbol)?;
            }
        }

        // Draw player (always visible)
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
        let fullness_display = state.player.fullness / 10;
        let max_fullness_display = state.player.max_fullness / 10;

        let weapon_str = state.player.weapon.as_ref()
            .map(|w| w.display_name())
            .unwrap_or_else(|| "-".into());
        let shield_str = state.player.shield.as_ref()
            .map(|s| s.display_name())
            .unwrap_or_else(|| "-".into());

        write!(
            self.stdout,
            "HP:{}/{}  Lv:{}  Exp:{}/{}  満腹度:{}/{}  武:{} 盾:{} F:{}/{}     ",
            state.player.hp,
            state.player.max_hp,
            state.player.level,
            state.player.exp,
            next_exp,
            fullness_display,
            max_fullness_display,
            weapon_str,
            shield_str,
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

    fn render_inventory(
        &mut self,
        items: &[Item],
        weapon: Option<&str>,
        shield: Option<&str>,
    ) -> Result<Option<InventoryAction>, Self::Error> {
        queue!(self.stdout, Clear(ClearType::All), MoveTo(0, 0))?;

        writeln!(self.stdout, "=== 持ち物 ({}/{}) ===", items.len(), 20)?;
        writeln!(self.stdout)?;

        if let Some(w) = weapon {
            writeln!(self.stdout, "  [装備中] 武器: {}", w)?;
        }
        if let Some(s) = shield {
            writeln!(self.stdout, "  [装備中] 盾:   {}", s)?;
        }
        if weapon.is_some() || shield.is_some() {
            writeln!(self.stdout)?;
        }

        if items.is_empty() {
            writeln!(self.stdout, "  (何も持っていない)")?;
        } else {
            for (i, item) in items.iter().enumerate() {
                let key = (b'a' + i as u8) as char;
                let charges_str = if let ItemCategory::Staff(c) = &item.category {
                    format!("[{}]", c)
                } else {
                    String::new()
                };
                writeln!(self.stdout, "  {} ) {} {}{}", key, item.symbol, item.name, charges_str)?;
            }
        }

        writeln!(self.stdout)?;
        writeln!(self.stdout, "  使うアイテムを選択 (a-{}) / ESC: 戻る",
            if items.is_empty() { '-' } else { (b'a' + (items.len() as u8).saturating_sub(1)) as char }
        )?;
        self.stdout.flush()?;

        // Wait for input
        loop {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('i') => {
                            return Ok(Some(InventoryAction::Cancel));
                        }
                        KeyCode::Char(c) if c >= 'a' => {
                            let idx = (c as u8 - b'a') as usize;
                            if idx < items.len() {
                                return Ok(Some(InventoryAction::Use(idx)));
                            }
                        }
                        _ => continue,
                    }
                }
                _ => continue,
            }
        }
    }

    fn render_throw_inventory(
        &mut self,
        items: &[Item],
        weapon: Option<&str>,
        shield: Option<&str>,
    ) -> Result<Option<InventoryAction>, Self::Error> {
        queue!(self.stdout, Clear(ClearType::All), MoveTo(0, 0))?;

        writeln!(self.stdout, "=== 投げるアイテムを選択 ({}/{}) ===", items.len(), 20)?;
        writeln!(self.stdout)?;

        if let Some(w) = weapon {
            writeln!(self.stdout, "  [装備中] 武器: {}", w)?;
        }
        if let Some(s) = shield {
            writeln!(self.stdout, "  [装備中] 盾:   {}", s)?;
        }
        if weapon.is_some() || shield.is_some() {
            writeln!(self.stdout)?;
        }

        if items.is_empty() {
            writeln!(self.stdout, "  (何も持っていない)")?;
        } else {
            for (i, item) in items.iter().enumerate() {
                let key = (b'a' + i as u8) as char;
                let charges_str = if let ItemCategory::Staff(c) = &item.category {
                    format!("[{}]", c)
                } else {
                    String::new()
                };
                writeln!(self.stdout, "  {} ) {} {}{}", key, item.symbol, item.name, charges_str)?;
            }
        }

        writeln!(self.stdout)?;
        writeln!(self.stdout, "  投げるアイテムを選択 (a-{}) / ESC: 戻る",
            if items.is_empty() { '-' } else { (b'a' + (items.len() as u8).saturating_sub(1)) as char }
        )?;
        self.stdout.flush()?;

        loop {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('t') => {
                            return Ok(Some(InventoryAction::Cancel));
                        }
                        KeyCode::Char(c) if c >= 'a' => {
                            let idx = (c as u8 - b'a') as usize;
                            if idx < items.len() {
                                return Ok(Some(InventoryAction::Throw(idx)));
                            }
                        }
                        _ => continue,
                    }
                }
                _ => continue,
            }
        }
    }

    fn render_game_over(&mut self, state: &GameState) -> Result<(), Self::Error> {
        queue!(self.stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        writeln!(self.stdout)?;
        writeln!(self.stdout, "        GAME OVER")?;
        writeln!(self.stdout)?;
        writeln!(self.stdout, "   到達フロア: {}/{}", state.floor, state.max_floor)?;
        writeln!(self.stdout, "   レベル: {}", state.player.level)?;
        writeln!(self.stdout, "   撃破数: {}", state.player.kill_count)?;
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
        writeln!(self.stdout, "   撃破数: {}", state.player.kill_count)?;
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
