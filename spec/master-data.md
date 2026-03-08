# マスターデータ設計

## 概要

ゲームの構成要素（モンスター・アイテム・ダンジョン・バランス等）をRONファイルで外部管理し、マスターデータの入れ替えで異なるローグライクゲームを再現可能にする。

## フォーマット: RON (Rusty Object Notation)

Rustの型と1:1で対応し、enumがそのまま記述できるため採用。

依存: `serde`, `ron`

## ディレクトリ構成

```
data/
└── games/
    └── default/                   // ゲームタイトル単位
        ├── game.ron               // ゲーム全体設定
        ├── player.ron             // プレイヤー初期パラメータ・成長テーブル
        ├── balance.ron            // スケーリング・ダメージ式パラメータ
        ├── messages.ron           // メッセージテンプレート
        ├── monsters/
        │   ├── _index.ron         // モンスター一覧
        │   ├── slime.ron          // 個別モンスター定義
        │   └── ...
        ├── items/
        │   ├── _index.ron         // アイテム一覧
        │   ├── herb.ron           // 個別アイテム定義
        │   └── ...
        └── floors/
            ├── _index.ron         // 全フロア共通設定
            └── ...                // フロア個別設定（必要に応じて）
```

## データ定義例

### モンスター定義

```ron
// data/games/default/monsters/slime.ron
MonsterDef(
    id: "slime",
    name: "スライム",
    symbol: 'M',
    base_hp: 5,
    base_attack: 2,
    base_defense: 3,
    base_exp: 4,
    ai_type: Standard(detection_range: 10),
    special_abilities: [],
    drop_table: [],
)
```

### アイテム定義

```ron
// data/games/default/items/herb.ron
ItemDef(
    id: "herb",
    name: "回復草",
    symbol: '!',
    category: Consumable,
    effect: Heal(25),
    buy_price: 50,
    sell_price: 25,
)
```

### フロア設定

```ron
// data/games/default/floors/_index.ron
FloorTable(
    entries: [
        FloorRange(
            floors: (1, 3),
            monster_count: (6, 8),
            monster_pool: [
                Spawn(id: "slime", weight: 10),
                Spawn(id: "goblin", weight: 5),
            ],
            item_spawns: [
                ItemSpawn(id: "herb", count: 3),
            ],
        ),
        // ...
    ],
)
```

### プレイヤー初期パラメータ・成長テーブル

```ron
// data/games/default/player.ron
PlayerConfig(
    initial_hp: 30,
    initial_attack: 8,
    initial_defense: 5,
    level_table: [
        // (累計経験値, 最大HP, 攻撃力, 防御力)
        (0, 30, 8, 5),       // Lv 1
        (30, 35, 10, 6),     // Lv 2
        (70, 40, 12, 7),     // Lv 3
        (120, 45, 14, 8),    // Lv 4
        (200, 50, 16, 9),    // Lv 5
        // ...
    ],
)
```

## コードとデータの境界

| マスターデータ（What） | コード（How） |
|----------------------|--------------|
| モンスターの名前・ステータス・出現階 | ターン進行のシーケンス制御 |
| アイテムの名前・効果・出現率 | ダンジョン生成アルゴリズム（BSP等） |
| ダンジョンの階数・構成・ルール | AI行動選択のフレームワーク |
| ダメージ計算のパラメータ | 衝突判定・移動制御 |
| レベルアップテーブル | 描画・入力処理 |
| メッセージテンプレート | セーブ/ロードの仕組み |
| 状態異常の種類・効果・持続ターン | インベントリ管理のロジック |

### 設計原則

**効果（Effect）をデータで定義し、エンジン側は効果の「実行器」に徹する。**

例: 「かなしばりの杖」をデータで追加するとき、コード側に必要なのは `apply_status("paralysis")` の実行能力だけ。アイテム固有の知識はコードに不要。

## データローダー

```rust
pub struct GameData {
    pub monsters: HashMap<String, MonsterDef>,
    pub items: HashMap<String, ItemDef>,
    pub floors: Vec<FloorConfig>,
    pub balance: BalanceConfig,
    pub player_config: PlayerConfig,
}

impl GameData {
    pub fn load(game_dir: &Path) -> Result<Self, DataLoadError>;
}
```
