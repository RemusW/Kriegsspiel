//! The board document: the maps laid on the table, the units on them, recorded turns, and
//! each umpire's undo history.
//!
//! `Board`, `MapImage`, `Unit`, `Pawn`, `UnitKind` and `Side` derive `TS`: running the tests
//! writes their TypeScript definitions to `client/src/bindings/`.
//!
//! An undo step records every unit and map it touched before and after. Undoing writes the
//! old values back only where the board still holds the values that step left, so later
//! changes by other umpires are kept.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Undo steps kept per umpire.
const UNDO_LIMIT: usize = 200;

/// The smallest and largest `Pawn::scale` a command may set.
pub const MIN_SCALE: f32 = 0.25;
pub const MAX_SCALE: f32 = 4.0;

/// World width a map image is given as it is laid on the table. Its depth follows the
/// image's own aspect ratio, so the width is the only thing that has to be chosen. Matches
/// `BLANK_TABLE_WIDTH` in `client/src/engine/map/MapTable.ts`, the bare table's size.
pub const NEW_MAP_WIDTH: f32 = 100.0;

/// Gap left between a new map and the maps already on the table, in world units.
const NEW_MAP_GAP: f32 = 10.0;

/// Keys an umpire's undo history and turn view.
pub type UmpireId = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "model.ts")]
pub enum UnitKind {
    Infantry,
    Cavalry,
    Artillery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "model.ts")]
pub enum Side {
    Blue,
    Red,
}

/// One image laid on the table. A game may have several at once: a theatre map with town
/// plans beside it, or two survey sheets covering one front.
///
/// The sheet's extent is held here rather than worked out from the image by each client, so
/// every client agrees on where its edges are and can draw it at the right shape before the
/// image has finished loading.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "model.ts")]
pub struct MapImage {
    /// Stable across snapshots, from the same counter as the units', and never reused.
    pub id: u64,
    /// Asset id of the image. Never `None`, unlike a unit's: a map is its image.
    pub asset: String,
    /// Centre of the sheet on the table (world X, world Z).
    pub pos: [f32; 2],
    /// Extent of the sheet (world X, world Z). Only the server sets it, from the pixel size
    /// in the image's own header, so a map always keeps its image's aspect ratio.
    pub size: [f32; 2],
}

/// Where a piece sits on the table and how its block is drawn.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "model.ts")]
pub struct Pawn {
    /// Position on the map plane (world X, world Z).
    pub pos: [f32; 2],
    /// Rotation around the vertical axis, in radians.
    pub rot: f32,
    pub scale: f32,
    /// Asset id of a custom top-face image. `None` uses the generated symbol.
    pub image: Option<String>,
}

/// A unit on the board. Its pawn fields sit at the top level of its JSON.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "model.ts")]
pub struct Unit {
    /// Stable across snapshots, and never reused within a game.
    pub id: u64,
    pub kind: UnitKind,
    pub side: Side,
    #[serde(flatten)]
    pub pawn: Pawn,
}

/// Everything that makes up one state of the table.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "model.ts")]
pub struct Board {
    /// The images laid on the table, bottom to top: where two sheets overlap, the later one
    /// is drawn over the earlier.
    // Defaulted when missing, which is what lets a format 1 save, written before a board
    // could hold more than one map, deserialize at all; `save.rs` then lays its single
    // `mapImage` down. The server always sends the field.
    #[serde(default)]
    pub maps: Vec<MapImage>,
    pub pawns: Vec<Unit>,
    /// Issues ids for everything on the board, maps and units alike.
    pub next_id: u64,
}

impl Board {
    pub fn spawn(&mut self, kind: UnitKind, side: Side, pos: [f32; 2]) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.pawns.push(Unit {
            id,
            kind,
            side,
            pawn: Pawn {
                pos,
                rot: 0.0,
                scale: 1.0,
                image: None,
            },
        });
        id
    }

    /// Lay an image on the table and return the new map's id. `pixels` is the image's size
    /// in pixels, read from its header, which is what keeps the sheet in the image's own
    /// aspect ratio.
    ///
    /// The sheet goes east of every map already down, clear of them all: a sheet dropped on
    /// top of another would be invisible, and nothing moves maps yet.
    pub fn add_map(&mut self, asset: String, pixels: (u32, u32)) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        // A zero dimension never reaches here, since the store refuses such an image, but
        // the arithmetic must not divide by zero even so.
        let (width, height) = (pixels.0.max(1) as f32, pixels.1.max(1) as f32);
        let size = [NEW_MAP_WIDTH, NEW_MAP_WIDTH * height / width];
        let x = match self.east_edge() {
            Some(edge) => edge + NEW_MAP_GAP + size[0] / 2.0,
            None => 0.0,
        };
        self.maps.push(MapImage {
            id,
            asset,
            pos: [x, 0.0],
            size,
        });
        id
    }

    /// Take these maps off the table. Ids that are not on it are ignored.
    pub fn remove_maps(&mut self, ids: &[u64]) {
        self.maps.retain(|map| !ids.contains(&map.id));
    }

    /// The furthest east any map reaches, or `None` on a bare table.
    fn east_edge(&self) -> Option<f32> {
        self.maps
            .iter()
            .map(|map| map.pos[0] + map.size[0] / 2.0)
            .reduce(f32::max)
    }

    /// The unit with this id, if it is on the board.
    pub fn unit_mut(&mut self, id: u64) -> Option<&mut Unit> {
        self.pawns.iter_mut().find(|unit| unit.id == id)
    }

    /// Every asset id this board names: each map's image, and any unit's top-face image.
    /// What a save file has to carry, and what may not be thrown away while this board
    /// exists.
    pub fn images(&self) -> impl Iterator<Item = &str> {
        self.maps
            .iter()
            .map(|map| map.asset.as_str())
            .chain(self.pawns.iter().filter_map(|unit| unit.pawn.image.as_deref()))
    }
}

/// One recorded turn: the board as it stood, and when an umpire recorded it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Turn {
    pub board: Board,
    /// Milliseconds since the Unix epoch. The caller supplies it, so the document needs no
    /// clock of its own and its tests stay deterministic.
    pub recorded_at: u64,
}

/// The outcome of an undo or redo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// There was nothing to undo or redo.
    Empty,
    /// The umpire is viewing a recorded turn, which is read-only. Nothing changed.
    ReadOnly,
    /// The whole step was applied.
    Applied,
    /// Some or all of the step was skipped because another umpire has changed the same
    /// things since.
    Partial,
}

/// The live board, the recorded turns, and each umpire's history and view.
#[derive(Debug, Default)]
pub struct Doc {
    /// The live board. Change it through [`Doc::edit`] so the change can be undone.
    pub board: Board,
    pub turns: Vec<Turn>,
    umpires: HashMap<UmpireId, Umpire>,
}

/// What the document keeps for one umpire.
#[derive(Debug, Default)]
struct Umpire {
    undo: Vec<Edit>,
    redo: Vec<Edit>,
    /// When set, this umpire sees a recorded turn, read-only, instead of the live board.
    viewing: Option<usize>,
}

impl Doc {
    /// What this umpire should see: the recorded turn they are viewing, or the live board.
    pub fn shown(&self, umpire: UmpireId) -> &Board {
        match self.viewing(umpire) {
            Some(i) => &self.turns[i].board,
            None => &self.board,
        }
    }

    /// The recorded turn this umpire is viewing, if any.
    pub fn viewing(&self, umpire: UmpireId) -> Option<usize> {
        self.umpires.get(&umpire).and_then(|u| u.viewing)
    }

    /// Apply one umpire's change to the live board as a single undo step for that umpire.
    /// Clears that umpire's redo, not anyone else's. Refused, returning false, while the
    /// umpire is viewing a recorded turn.
    pub fn edit(&mut self, umpire: UmpireId, change: impl FnOnce(&mut Board)) -> bool {
        if self.viewing(umpire).is_some() {
            return false;
        }
        let before = self.board.clone();
        change(&mut self.board);
        // Ids are never reused, even when a restored turn carries a lower counter: undo steps
        // may still refer to them.
        self.board.next_id = self.board.next_id.max(before.next_id);

        let edit = Edit::between(&before, &self.board);
        if !edit.is_empty() {
            let me = self.umpires.entry(umpire).or_default();
            push_capped(&mut me.undo, edit);
            me.redo.clear();
        }
        true
    }

    /// Undo this umpire's latest edit, leaving other umpires' later changes in place.
    pub fn undo(&mut self, umpire: UmpireId) -> Step {
        self.step(umpire, true)
    }

    /// Redo this umpire's latest undo.
    pub fn redo(&mut self, umpire: UmpireId) -> Step {
        self.step(umpire, false)
    }

    fn step(&mut self, umpire: UmpireId, backwards: bool) -> Step {
        let me = self.umpires.entry(umpire).or_default();
        if me.viewing.is_some() {
            return Step::ReadOnly;
        }
        let from = if backwards { &mut me.undo } else { &mut me.redo };
        let Some(edit) = from.pop() else {
            return Step::Empty;
        };

        let before = self.board.clone();
        let skipped = edit.apply(&mut self.board, backwards);

        // Keep what actually changed, so redoing a partial undo repeats only that part.
        // Stored edits always point forwards: `before` is the older state.
        let (done, to) = if backwards {
            (Edit::between(&self.board, &before), &mut me.redo)
        } else {
            (Edit::between(&before, &self.board), &mut me.undo)
        };
        if !done.is_empty() {
            push_capped(to, done);
        }

        if skipped { Step::Partial } else { Step::Applied }
    }

    /// Record the live board as a new turn, stamped with `recorded_at` (milliseconds since
    /// the Unix epoch). Returns the number of recorded turns.
    pub fn record_turn(&mut self, recorded_at: u64) -> usize {
        self.turns.push(Turn {
            board: self.board.clone(),
            recorded_at,
        });
        self.turns.len()
    }

    /// Step this umpire through recorded turns. Stepping back from the live board goes to
    /// the last turn; stepping forward past the last turn returns to the live board.
    pub fn step_turn(&mut self, umpire: UmpireId, forward: bool) {
        let n = self.turns.len();
        let me = self.umpires.entry(umpire).or_default();
        me.viewing = match (me.viewing, forward) {
            (None, false) if n > 0 => Some(n - 1),
            (None, _) => None,
            (Some(i), false) => Some(i.saturating_sub(1)),
            (Some(i), true) if i + 1 < n => Some(i + 1),
            (Some(_), true) => None,
        };
    }

    /// Show this umpire one recorded turn, or the live board with `None`. Refused, returning
    /// false, for a turn that does not exist; the view is then left as it was.
    pub fn view_turn(&mut self, umpire: UmpireId, turn: Option<usize>) -> bool {
        if turn.is_some_and(|i| i >= self.turns.len()) {
            return false;
        }
        self.umpires.entry(umpire).or_default().viewing = turn;
        true
    }

    /// Make the turn this umpire is viewing the live board, as one undoable edit.
    pub fn restore_viewed_turn(&mut self, umpire: UmpireId) -> bool {
        let Some(i) = self.umpires.get_mut(&umpire).and_then(|u| u.viewing.take()) else {
            return false;
        };
        let board = self.turns[i].board.clone();
        self.edit(umpire, |b| *b = board)
    }
}

/// One unit before and after an edit. `None` means the unit did not exist.
#[derive(Clone, Debug, PartialEq)]
struct UnitChange {
    id: u64,
    before: Option<Unit>,
    after: Option<Unit>,
}

/// One map before and after an edit. `None` means the map was not on the table. `at` is the
/// place it had in the board's list, so bringing one back puts it at the same height in the
/// stack rather than on top.
#[derive(Clone, Debug, PartialEq)]
struct MapChange {
    id: u64,
    at: usize,
    before: Option<MapImage>,
    after: Option<MapImage>,
}

/// Everything one edit changed: one undo step.
#[derive(Clone, Debug, Default, PartialEq)]
struct Edit {
    units: Vec<UnitChange>,
    maps: Vec<MapChange>,
}

impl Edit {
    /// What changed from one board to another. The id counter is not tracked: undo never
    /// winds it back.
    fn between(before: &Board, after: &Board) -> Self {
        let old: HashMap<u64, &Unit> = before.pawns.iter().map(|u| (u.id, u)).collect();
        let new: HashMap<u64, &Unit> = after.pawns.iter().map(|u| (u.id, u)).collect();

        // Walk the boards' own unit lists, not the maps, so the order is deterministic.
        let mut units = Vec::new();
        for unit in &before.pawns {
            let now = new.get(&unit.id).copied();
            if now != Some(unit) {
                units.push(UnitChange {
                    id: unit.id,
                    before: Some(unit.clone()),
                    after: now.cloned(),
                });
            }
        }
        for unit in &after.pawns {
            if !old.contains_key(&unit.id) {
                units.push(UnitChange {
                    id: unit.id,
                    before: None,
                    after: Some(unit.clone()),
                });
            }
        }

        // The same walk again for the maps, which are a list with ids like the units, except
        // that their order is the order they are drawn in and so is worth putting back.
        let old_maps: HashMap<u64, &MapImage> = before.maps.iter().map(|m| (m.id, m)).collect();
        let new_maps: HashMap<u64, &MapImage> = after.maps.iter().map(|m| (m.id, m)).collect();

        let mut maps = Vec::new();
        for (at, map) in before.maps.iter().enumerate() {
            let now = new_maps.get(&map.id).copied();
            if now != Some(map) {
                maps.push(MapChange {
                    id: map.id,
                    at,
                    before: Some(map.clone()),
                    after: now.cloned(),
                });
            }
        }
        for (at, map) in after.maps.iter().enumerate() {
            if !old_maps.contains_key(&map.id) {
                maps.push(MapChange {
                    id: map.id,
                    at,
                    before: None,
                    after: Some(map.clone()),
                });
            }
        }

        Self { units, maps }
    }

    fn is_empty(&self) -> bool {
        self.units.is_empty() && self.maps.is_empty()
    }

    /// Move the board across this edit: backwards to undo it, forwards to redo it. A value
    /// moves only where the board still holds what the edit left there. Returns whether
    /// anything was skipped because someone changed it since.
    fn apply(&self, board: &mut Board, backwards: bool) -> bool {
        let mut skipped = false;

        for change in &self.units {
            let (from, to) = if backwards {
                (&change.after, &change.before)
            } else {
                (&change.before, &change.after)
            };
            let index = board.pawns.iter().position(|u| u.id == change.id);
            match (index, from, to) {
                // Remove a unit the edit created, unless someone has changed it since.
                (Some(i), Some(from), None) if board.pawns[i] == *from => {
                    board.pawns.remove(i);
                }
                // Bring back a unit the edit removed.
                (None, None, Some(to)) => board.pawns.push(to.clone()),
                // Change back each field nobody else has touched since.
                (Some(i), Some(from), Some(to)) => {
                    skipped |= merge_fields(&mut board.pawns[i], from, to);
                }
                // Someone else has since changed, removed or brought back this unit.
                _ => skipped = true,
            }
        }

        for change in &self.maps {
            let (from, to) = if backwards {
                (&change.after, &change.before)
            } else {
                (&change.before, &change.after)
            };
            let index = board.maps.iter().position(|map| map.id == change.id);
            match (index, from, to) {
                // Take a map off again, unless someone has changed it since.
                (Some(i), Some(from), None) if board.maps[i] == *from => {
                    board.maps.remove(i);
                }
                // Put a map back where it was in the stack. Clamped, because maps that were
                // below it may have been taken off since.
                (None, None, Some(to)) => {
                    board.maps.insert(change.at.min(board.maps.len()), to.clone());
                }
                // Change back each field nobody else has touched since.
                (Some(i), Some(from), Some(to)) => {
                    skipped |= merge_map(&mut board.maps[i], from, to);
                }
                // Someone else has since changed, removed or brought back this map.
                _ => skipped = true,
            }
        }
        skipped
    }
}

/// Move each field of `unit` that still holds its value in `from` to its value in `to`.
/// Returns whether any field was left alone.
fn merge_fields(unit: &mut Unit, from: &Unit, to: &Unit) -> bool {
    // Every field is named, with no `..`, so adding a field to `Unit` fails to compile
    // until it is handled here.
    let Unit {
        id: _,
        kind,
        side,
        pawn:
            Pawn {
                pos,
                rot,
                scale,
                image,
            },
    } = to;
    let skipped = [
        merge(&mut unit.kind, &from.kind, kind),
        merge(&mut unit.side, &from.side, side),
        merge(&mut unit.pawn.pos, &from.pawn.pos, pos),
        merge(&mut unit.pawn.rot, &from.pawn.rot, rot),
        merge(&mut unit.pawn.scale, &from.pawn.scale, scale),
        merge(&mut unit.pawn.image, &from.pawn.image, image),
    ];
    skipped.contains(&true)
}

/// Move each field of `map` that still holds its value in `from` to its value in `to`.
/// Returns whether any field was left alone.
fn merge_map(map: &mut MapImage, from: &MapImage, to: &MapImage) -> bool {
    // Named with no `..`, so a new field on `MapImage` fails to compile until it is handled.
    let MapImage {
        id: _,
        asset,
        pos,
        size,
    } = to;
    let skipped = [
        merge(&mut map.asset, &from.asset, asset),
        merge(&mut map.pos, &from.pos, pos),
        merge(&mut map.size, &from.size, size),
    ];
    skipped.contains(&true)
}

/// Set `field` to `to` if it still holds `from`. Returns true when it was left alone because
/// someone else set it to a third value.
fn merge<T: PartialEq + Clone>(field: &mut T, from: &T, to: &T) -> bool {
    if field == from {
        *field = to.clone();
        false
    } else {
        field != to
    }
}

fn push_capped(stack: &mut Vec<Edit>, edit: Edit) {
    stack.push(edit);
    if stack.len() > UNDO_LIMIT {
        stack.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: UmpireId = 1;
    const B: UmpireId = 2;

    /// A document with units 0 and 1, placed outside any umpire's history.
    fn two_units() -> Doc {
        let mut doc = Doc::default();
        doc.board.spawn(UnitKind::Infantry, Side::Blue, [0.0, 0.0]);
        doc.board.spawn(UnitKind::Cavalry, Side::Red, [10.0, 0.0]);
        doc
    }

    fn unit(board: &mut Board, id: u64) -> &mut Unit {
        board.pawns.iter_mut().find(|u| u.id == id).unwrap()
    }

    fn pos(doc: &Doc, id: u64) -> [f32; 2] {
        doc.board.pawns.iter().find(|u| u.id == id).unwrap().pawn.pos
    }

    #[test]
    fn undo_redo_round_trip() {
        let mut doc = Doc::default();
        doc.edit(A, |b| {
            b.spawn(UnitKind::Infantry, Side::Blue, [0.0, 0.0]);
        });
        assert_eq!(doc.undo(A), Step::Applied);
        assert!(doc.board.pawns.is_empty());
        assert_eq!(doc.redo(A), Step::Applied);
        assert_eq!(doc.board.pawns.len(), 1);
        assert_eq!(doc.redo(A), Step::Empty);
    }

    #[test]
    fn undo_reverts_only_your_own_edit() {
        let mut doc = two_units();
        doc.edit(B, |b| unit(b, 0).pawn.pos = [1.0, 1.0]);
        doc.edit(A, |b| unit(b, 1).pawn.pos = [2.0, 2.0]);

        assert_eq!(doc.undo(B), Step::Applied);
        assert_eq!(pos(&doc, 0), [0.0, 0.0]);
        assert_eq!(pos(&doc, 1), [2.0, 2.0]);
    }

    #[test]
    fn undo_skips_fields_another_umpire_changed_since() {
        let mut doc = two_units();
        doc.edit(B, |b| {
            let u = unit(b, 0);
            u.pawn.pos = [1.0, 1.0];
            u.pawn.rot = 0.5;
        });
        doc.edit(A, |b| unit(b, 0).pawn.pos = [2.0, 2.0]);

        assert_eq!(doc.undo(B), Step::Partial);
        let u = unit(&mut doc.board, 0);
        assert_eq!(u.pawn.pos, [2.0, 2.0], "A's later move stays");
        assert_eq!(u.pawn.rot, 0.0, "B's rotation is undone");
    }

    #[test]
    fn redo_after_partial_undo_repeats_only_what_was_undone() {
        let mut doc = two_units();
        doc.edit(B, |b| {
            let u = unit(b, 0);
            u.pawn.pos = [1.0, 1.0];
            u.pawn.rot = 0.5;
        });
        doc.edit(A, |b| unit(b, 0).pawn.pos = [2.0, 2.0]);
        doc.undo(B);

        assert_eq!(doc.redo(B), Step::Applied);
        let u = unit(&mut doc.board, 0);
        assert_eq!(u.pawn.pos, [2.0, 2.0]);
        assert_eq!(u.pawn.rot, 0.5);
    }

    #[test]
    fn undoing_a_spawn_keeps_the_unit_if_someone_changed_it() {
        let mut doc = Doc::default();
        doc.edit(B, |b| {
            b.spawn(UnitKind::Infantry, Side::Blue, [0.0, 0.0]);
        });
        doc.edit(A, |b| unit(b, 0).pawn.pos = [5.0, 5.0]);

        assert_eq!(doc.undo(B), Step::Partial);
        assert_eq!(pos(&doc, 0), [5.0, 5.0]);
    }

    #[test]
    fn undoing_a_delete_brings_the_unit_back() {
        let mut doc = two_units();
        doc.edit(A, |b| b.pawns.retain(|u| u.id != 0));
        assert_eq!(doc.undo(A), Step::Applied);
        assert_eq!(pos(&doc, 0), [0.0, 0.0]);
    }

    #[test]
    fn an_edit_clears_only_its_own_umpires_redo() {
        let mut doc = two_units();
        doc.edit(A, |b| unit(b, 0).pawn.pos = [1.0, 1.0]);
        doc.undo(A);
        doc.edit(B, |b| unit(b, 1).pawn.pos = [2.0, 2.0]);
        assert_eq!(doc.redo(A), Step::Applied);

        doc.undo(A);
        doc.edit(A, |b| unit(b, 1).pawn.rot = 1.0);
        assert_eq!(doc.redo(A), Step::Empty);
    }

    #[test]
    fn ids_are_never_reused() {
        let mut doc = Doc::default();
        doc.record_turn(0);
        doc.edit(A, |b| {
            b.spawn(UnitKind::Infantry, Side::Blue, [0.0, 0.0]);
        });
        doc.undo(A);
        doc.edit(A, |b| {
            b.spawn(UnitKind::Infantry, Side::Blue, [0.0, 0.0]);
        });
        assert_eq!(doc.board.pawns[0].id, 1, "undo does not wind back the counter");

        doc.step_turn(A, false);
        assert!(doc.restore_viewed_turn(A));
        doc.edit(A, |b| {
            b.spawn(UnitKind::Cavalry, Side::Red, [1.0, 1.0]);
        });
        assert_eq!(doc.board.pawns[0].id, 2, "restoring a turn does not either");
    }

    #[test]
    fn turn_viewing_is_per_umpire_and_read_only() {
        let mut doc = Doc::default();
        doc.record_turn(0);
        doc.board.spawn(UnitKind::Cavalry, Side::Red, [1.0, 1.0]);
        doc.record_turn(0);
        doc.board.spawn(UnitKind::Artillery, Side::Red, [1.0, 1.0]);

        doc.step_turn(A, false);
        assert_eq!(doc.viewing(A), Some(1));
        assert_eq!(doc.viewing(B), None);
        assert_eq!(doc.shown(B).pawns.len(), 2);
        assert!(!doc.edit(A, |b| b.pawns.clear()));
        assert_eq!(doc.undo(A), Step::ReadOnly);
        assert!(doc.edit(B, |b| unit(b, 0).pawn.rot = 1.0));

        doc.step_turn(A, false);
        assert_eq!(doc.viewing(A), Some(0));
        assert!(doc.shown(A).pawns.is_empty());
        doc.step_turn(A, true);
        doc.step_turn(A, true);
        assert_eq!(doc.viewing(A), None);
        assert_eq!(doc.shown(A).pawns.len(), 2);
    }

    #[test]
    fn viewing_jumps_to_a_turn_and_refuses_one_that_does_not_exist() {
        let mut doc = Doc::default();
        doc.record_turn(1000);
        doc.board.spawn(UnitKind::Cavalry, Side::Red, [1.0, 1.0]);
        doc.record_turn(2000);

        assert!(doc.view_turn(A, Some(0)));
        assert_eq!(doc.viewing(A), Some(0));
        assert!(doc.shown(A).pawns.is_empty());

        assert!(!doc.view_turn(A, Some(2)), "no third turn to look at");
        assert_eq!(doc.viewing(A), Some(0), "the view is left as it was");

        assert!(doc.view_turn(A, None));
        assert_eq!(doc.viewing(A), None);
        assert_eq!(doc.turns[1].recorded_at, 2000);
    }

    #[test]
    fn restoring_a_turn_is_one_undo_step() {
        let mut doc = Doc::default();
        doc.board.spawn(UnitKind::Infantry, Side::Blue, [0.0, 0.0]);
        doc.record_turn(0);
        doc.board.spawn(UnitKind::Cavalry, Side::Red, [1.0, 1.0]);

        doc.step_turn(A, false);
        assert!(doc.restore_viewed_turn(A));
        assert_eq!(doc.viewing(A), None);
        assert_eq!(doc.board.pawns.len(), 1);
        assert_eq!(doc.undo(A), Step::Applied);
        assert_eq!(doc.board.pawns.len(), 2);
    }

    #[test]
    fn undo_history_is_capped() {
        let mut doc = two_units();
        for i in 0..=UNDO_LIMIT {
            doc.edit(A, |b| unit(b, 0).pawn.rot = i as f32 + 1.0);
        }
        for _ in 0..UNDO_LIMIT {
            assert_eq!(doc.undo(A), Step::Applied);
        }
        assert_eq!(doc.undo(A), Step::Empty);
    }

    #[test]
    fn a_new_map_keeps_its_image_s_shape_and_stays_clear_of_the_others() {
        let mut board = Board::default();
        board.add_map("wide".into(), (4000, 2000));
        assert_eq!(board.maps[0].size, [NEW_MAP_WIDTH, NEW_MAP_WIDTH / 2.0]);
        assert_eq!(board.maps[0].pos, [0.0, 0.0], "the first map is centred");

        // A portrait sheet beside it: taller than wide, and east of the first with a gap.
        board.add_map("tall".into(), (500, 1000));
        assert_eq!(board.maps[1].size, [NEW_MAP_WIDTH, NEW_MAP_WIDTH * 2.0]);
        let gap = board.maps[1].pos[0]
            - board.maps[1].size[0] / 2.0
            - (board.maps[0].pos[0] + board.maps[0].size[0] / 2.0);
        assert_eq!(gap, NEW_MAP_GAP);
        assert_ne!(board.maps[0].id, board.maps[1].id);
    }

    #[test]
    fn maps_and_units_are_given_ids_from_one_counter() {
        let mut board = Board::default();
        let first = board.add_map("map".into(), (10, 10));
        let unit = board.spawn(UnitKind::Infantry, Side::Blue, [0.0, 0.0]);
        let second = board.add_map("map".into(), (10, 10));
        assert_eq!([first, unit, second], [0, 1, 2]);
    }

    #[test]
    fn laying_a_map_down_and_taking_it_off_are_undoable() {
        let mut doc = Doc::default();
        doc.edit(A, |b| {
            b.add_map("first".into(), (100, 100));
        });
        doc.edit(A, |b| {
            b.add_map("second".into(), (100, 100));
        });
        let id = doc.board.maps[0].id;

        doc.edit(A, |b| b.remove_maps(&[id]));
        assert_eq!(doc.board.maps.len(), 1);

        // Undoing a removal puts the sheet back under the one that was laid over it, not on
        // top of it.
        assert_eq!(doc.undo(A), Step::Applied);
        assert_eq!(
            doc.board
                .maps
                .iter()
                .map(|m| m.asset.as_str())
                .collect::<Vec<_>>(),
            ["first", "second"]
        );

        assert_eq!(doc.undo(A), Step::Applied);
        assert_eq!(doc.board.maps.len(), 1, "the second map is taken back");
        assert_eq!(doc.undo(A), Step::Applied);
        assert!(doc.board.maps.is_empty(), "and so is the first");
        assert_eq!(doc.redo(A), Step::Applied);
        assert_eq!(doc.board.maps[0].asset, "first");
    }

    #[test]
    fn undoing_a_map_skips_what_another_umpire_moved_since() {
        let mut doc = Doc::default();
        doc.edit(A, |b| {
            b.add_map("map".into(), (100, 100));
        });
        doc.edit(B, |b| b.maps[0].pos = [25.0, 0.0]);

        // A's undo would take the map off, but B has since moved it: it stays.
        assert_eq!(doc.undo(A), Step::Partial);
        assert_eq!(doc.board.maps.len(), 1);
        assert_eq!(doc.board.maps[0].pos, [25.0, 0.0]);
    }

    #[test]
    fn board_json_uses_client_field_names() {
        let mut board = Board::default();
        board.add_map("map-1".into(), (200, 100));
        board.spawn(UnitKind::Infantry, Side::Blue, [3.0, -4.5]);

        let json = serde_json::to_value(&board).unwrap();
        assert_eq!(json["maps"][0]["asset"], "map-1");
        assert_eq!(json["maps"][0]["size"][1], NEW_MAP_WIDTH / 2.0);
        assert_eq!(json["nextId"], 2);
        assert_eq!(json["pawns"][0]["kind"], "Infantry");
        assert_eq!(json["pawns"][0]["pos"][1], -4.5);

        let back: Board = serde_json::from_value(json).unwrap();
        assert_eq!(back, board);
    }
}
