# アイテム（Phase 1: 基本）

## 概要

フロアに落ちているアイテムを拾い、使用できるシステム。Phase 1 では回復アイテムのみ。

## アイテム種別（Phase 1）

| アイテム | 記号 | 効果 | フロアあたりの出現数 |
|---------|------|------|-------------------|
| 薬草    | !    | HP を 25 回復（最大HPを超えない） | 3 |

※ アイテムの定義はマスターデータで管理する（[master-data.md](./master-data.md) 参照）。

## アイテムの配置

- フロア生成時に部屋の中のランダムな位置に配置
- 壁・階段・モンスター・プレイヤーの初期位置と重ならないこと
- マップの `MapCell.items` にアイテムIDを格納

## 拾う

- プレイヤーがアイテムのあるマスに移動すると自動で拾う
- インベントリ（最大20個）に追加される
- **インベントリが満杯の場合**: 拾わずにマスを通過できる。メッセージ「持ち物がいっぱいだ」を表示
- GameEvent::ItemPickedUp を発行

## 使う

- `i` キーでインベントリ表示 → 番号選択で使用
- 使用するとインベントリから消費される
- GameEvent::ItemUsed を発行

## データ構造

```rust
struct Item {
    id: String,         // マスターデータのID
    name: String,
    symbol: char,       // 表示用記号（1文字）
    effect: ItemEffect,
}

enum ItemEffect {
    Heal(i32),
}

// プレイヤーに追加
struct Player {
    // ... 既存フィールド
    inventory: Vec<Item>,  // 最大20個
}
```

## Phase 2 以降の拡張

- 武器・盾（装備システム → [equipment.md](./equipment.md)）
- 草・巻物・杖（多様なアイテム → [items-advanced.md](./items-advanced.md)）
- 未識別アイテム
- 投擲（[throwing.md](./throwing.md)）
- 壺
