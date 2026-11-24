// Exercise 0001 feedback loop. Paste the whole block inside the existing `mod tests` in
// `crates/server/src/lib.rs`, just above `fn placement_json_leaves_out_unset_fields`.
//
// Run with:  cargo test -p kriegsspiel-server presence
//            cargo test -p kriegsspiel-server          (the whole suite, and the bindings)
//
// Nothing here is the implementation. Every test calls names you have not written yet, so
// the first failure is a compile error naming what is missing. Work down the list.

/// A presence message holding one unit at a point on the map.
fn holding(id: u64, pos: [f32; 2]) -> String {
    format!(
        r#"{{"type":"presence","holding":[{{"id":{id},"pos":[{},{}]}}]}}"#,
        pos[0], pos[1]
    )
}

#[tokio::test]
async fn presence_is_not_an_edit() {
    let game = Game::demo();
    let changed = game.changed.subscribe();
    let presence = game.presence.subscribe();

    assert_eq!(game.receive(0, &holding(0, [1.0, 2.0])).await, Effect::None);
    // It reaches no board, so no client is sent one and the unit has not moved: a drag in
    // progress is not an edit until the umpire lets go.
    assert!(!changed.has_changed().unwrap());
    assert_eq!(game.doc.read().await.board.pawns[0].pawn.pos, [-15.0, -5.0]);
    assert!(presence.has_changed().unwrap(), "the others are told");
}

#[tokio::test]
async fn an_umpire_hears_only_the_other_umpires_holdings() {
    let game = Game::demo();
    game.receive(0, &holding(0, [1.0, 2.0])).await;
    game.receive(1, &holding(1, [3.0, 4.0])).await;

    let held = game.presence.borrow();
    // Umpire 0's own drag is already on its own screen; sending it back would fight it.
    assert_eq!(others_holding(&held, 0).len(), 1);
    assert_eq!(others_holding(&held, 0)[0].id, 1);
    assert_eq!(others_holding(&held, 2).len(), 2, "a third sees both");
}

#[tokio::test]
async fn an_unchanged_holding_wakes_nobody() {
    let game = Game::demo();
    game.receive(0, &holding(0, [1.0, 2.0])).await;

    let presence = game.presence.subscribe();
    game.receive(0, &holding(0, [1.0, 2.0])).await;
    assert!(!presence.has_changed().unwrap(), "nothing to say");

    game.receive(0, &holding(0, [1.0, 3.0])).await;
    assert!(presence.has_changed().unwrap(), "it moved");
}

#[tokio::test]
async fn an_umpire_holding_nothing_has_no_entry() {
    let game = Game::demo();
    game.receive(0, &holding(0, [1.0, 2.0])).await;
    game.receive(0, r#"{"type":"presence","holding":[]}"#).await;

    // An empty entry would tell a client that connects later about a drag already over.
    assert!(game.presence.borrow().is_empty());
}

#[tokio::test]
async fn presence_carrying_a_broken_value_is_dropped() {
    let game = Game::demo();
    let presence = game.presence.subscribe();

    // 1e39 does not fit an f32 and arrives as infinity, as in `Spawn`.
    let message = r#"{"type":"presence","holding":[{"id":0,"pos":[1e39,0]}]}"#;
    assert_eq!(game.receive(0, message).await, Effect::None);
    assert!(!presence.has_changed().unwrap());
    assert!(game.presence.borrow().is_empty());
}

#[tokio::test]
async fn a_finished_connection_holds_nothing() {
    let game = Game::demo();
    game.receive(0, &holding(0, [1.0, 2.0])).await;

    let presence = game.presence.subscribe();
    game.forget_presence(0);
    assert!(presence.has_changed().unwrap(), "the pieces are put down");
    assert!(game.presence.borrow().is_empty());

    // Nothing to say when an umpire that held nothing goes.
    let quiet = game.presence.subscribe();
    game.forget_presence(0);
    assert!(!quiet.has_changed().unwrap());
}
