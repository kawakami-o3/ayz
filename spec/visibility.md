# 視界システム

## 概要

部屋と通路で視界範囲が異なるシレン系の視界ルール。探索の緊張感を生む。

## 基本ルール

| 場所 | 視界範囲 |
|------|---------|
| 部屋内 | 部屋全体が見える |
| 通路 | プレイヤーの周囲1マス（8方向）のみ |

## 詳細仕様

### 部屋内

- プレイヤーが部屋にいる場合、同じ room_id を持つセルが全て可視
- 部屋内のモンスター・アイテム・階段が全て表示される
- 他の部屋は不可視（一度訪れた部屋は地形のみ記憶）

### 通路

- プレイヤーの周囲1マス（8方向）のみ可視
- 「角を曲がったらモンスターがいた」という緊張感を生む
- 通路にいるモンスターは隣接しないと見えない

### マップ記憶

- 一度訪れたマス（可視になったマス）は地形が記憶される
- 記憶されたマスは暗い色で表示（モンスター・アイテムは非表示）
- 未踏のマスは完全に非表示

## 描画の区別

| 状態 | 表示 |
|------|------|
| 現在可視 | 通常色で表示（モンスター・アイテム含む） |
| 記憶済み | 暗い色で地形のみ表示 |
| 未踏 | 非表示（空白） |

## 特殊アイテムとの連携

- 「あかりの巻物」使用時: フロア全体が可視になる（モンスター・アイテム位置も表示）
- 効果は現在のフロアにいる間持続

## 実装上の注意

- `GameMap` にプレイヤーの訪問済みマス情報を持たせる: `visited: Vec<Vec<bool>>`
- 毎ターンの可視判定は `room_id` ベースで効率的に計算可能
- Renderer に可視状態を渡し、描画を制御する

```rust
pub struct Visibility {
    pub visible: HashSet<Position>,   // 現在可視のマス
    pub visited: HashSet<Position>,   // 訪問済み（記憶）のマス
}

impl Visibility {
    pub fn update(&mut self, player_pos: &Position, map: &GameMap) {
        self.visible.clear();
        match map.get(player_pos).map(|c| &c.terrain) {
            Some(Terrain::Floor { room_id: Some(id) }) => {
                // 同じ room_id のセルを全て可視に
                self.reveal_room(map, *id);
            }
            _ => {
                // 周囲1マスのみ可視
                self.reveal_adjacent(player_pos);
            }
        }
        // 可視マスを訪問済みに追加
        self.visited.extend(self.visible.iter().cloned());
    }
}
```
