# アーキテクチャ

## 概要

ゲームロジックとUIの分離、データ駆動設計への対応、将来の2D GUI移行を見据えたモジュール構成を定義する。

## モジュール構成

```
src/
├── lib.rs                  // クレートルート
├── main.rs                 // エントリポイント（最小限）
│
├── core/                   // ゲームエンジンコア（UI非依存）
│   ├── mod.rs
│   ├── types.rs            // Position, Direction 等の基本型
│   ├── entity.rs           // Player, Monster, Item の定義
│   ├── dungeon.rs          // Dungeon 状態管理
│   ├── combat.rs           // 戦闘システム
│   ├── ai.rs               // モンスターAI
│   ├── inventory.rs        // インベントリ・アイテム使用
│   ├── growth.rs           // 経験値・レベルアップ
│   └── turn.rs             // ターン進行ロジック
│
├── map/                    // マップ関連
│   ├── mod.rs
│   ├── cell.rs             // セルの型定義
│   ├── generator.rs        // BSPダンジョン生成
│   └── spawn.rs            // エンティティ配置ロジック
│
├── data/                   // マスターデータのロード・管理
│   ├── mod.rs
│   ├── loader.rs           // ファイルI/O + デシリアライズ
│   ├── monster_data.rs     // モンスター定義
│   ├── item_data.rs        // アイテム定義
│   └── floor_data.rs       // フロア設定
│
├── ui/                     // UI層（trait + 実装）
│   ├── mod.rs
│   ├── renderer.rs         // Renderer trait 定義
│   ├── input.rs            // InputHandler trait 定義
│   └── terminal/           // crossterm 実装
│       ├── mod.rs
│       ├── renderer.rs
│       └── input.rs
│
└── app.rs                  // アプリケーション統合
```

## マップ表現の変更

### 現状の問題

マップが `Vec<String>` で管理されており、セルに複数の属性（部屋ID、アイテムの有無等）を持たせられない。モンスターAIの「同じ部屋判定」やアイテム配置に支障がある。

### 新しいセル表現

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terrain {
    Wall,
    Floor { room_id: Option<u8> },
    Aisle,
    Exit,
}

#[derive(Clone, Debug)]
pub struct MapCell {
    pub terrain: Terrain,
    pub items: Vec<ItemId>,
    pub trap: Option<TrapId>,     // Phase 3 で使用
}

pub struct GameMap {
    pub width: usize,
    pub height: usize,
    cells: Vec<Vec<MapCell>>,
    pub exit_pos: Position,
}
```

## Command / Event パターン

ゲームロジックは GameCommand を受け取り、GameEvent のリストを返す。UIはイベントを描画に変換する。

```
[InputHandler] → GameCommand → [GameState] → GameEvent(s) → [Renderer]
```

### GameCommand

```rust
pub enum GameCommand {
    Move(Direction),      // 移動 or 攻撃（移動先にモンスターがいれば攻撃）
    UseItem(usize),       // インベントリのインデックス
    Wait,                 // 足踏み（ターン消費）
    Quit,
}
```

### GameEvent

```rust
pub enum GameEvent {
    PlayerMoved { from: Position, to: Position },
    PlayerAttacked { target_name: String, damage: i32 },
    MonsterDefeated { name: String, exp: i32 },
    MonsterMoved { id: usize, from: Position, to: Position },
    MonsterAttacked { name: String, damage: i32 },
    PlayerDamaged { amount: i32, remaining_hp: i32 },
    ItemPickedUp { name: String },
    ItemUsed { name: String, effect_desc: String },
    LevelUp { new_level: i32 },
    FloorAdvance { new_floor: u32 },
    GameOver,
    GameClear,
    Message(String),
}
```

## UI 分離（Renderer / InputHandler trait）

```rust
pub trait Renderer {
    fn render(&mut self, state: &GameState) -> Result<(), RenderError>;
    fn push_message(&mut self, msg: &str);
    fn render_game_over(&mut self, state: &GameState) -> Result<(), RenderError>;
    fn render_game_clear(&mut self, state: &GameState) -> Result<(), RenderError>;
    fn render_inventory(&mut self, items: &[Item]) -> Result<Option<usize>, RenderError>;
    fn cleanup(&mut self) -> Result<(), RenderError>;
}

pub trait InputHandler {
    fn next_command(&mut self) -> Result<GameCommand, InputError>;
}
```

- CLI版: `TerminalRenderer` + `TerminalInput`（crossterm）
- 将来の2D版: `GraphicsRenderer` + `GraphicsInput`（bevy等）
- テスト: `MockRenderer` + 直接 GameCommand 生成

## アプリケーション統合

```rust
pub struct App<R: Renderer, I: InputHandler> {
    state: GameState,
    renderer: R,
    input: I,
}
```

`main.rs` では具象型を組み立てて `App::run()` を呼ぶだけにする。

## ターン制の定義

- 1ターン = プレイヤーが1回行動 → 全モンスターが1回ずつ行動
- モンスターの行動順序は配列の先頭から順
- 移動・攻撃・アイテム使用・足踏みの全てが1ターン消費
- 将来的に倍速・鈍足の速度差を導入可能な設計にする

## 乱数シード管理

リプレイやデバッグのためにシード管理を設ける。

```rust
pub struct RngManager {
    master_seed: u64,
    map_rng: StdRng,     // マップ生成用
    game_rng: StdRng,    // 戦闘・AI用
}
```
