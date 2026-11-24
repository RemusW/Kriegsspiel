# Each connection is sent its own view as a whole-board snapshot

After a change, a `tokio::sync::watch` channel wakes every connection, and each sends its own
view as a whole board. Umpires viewing different turns, and later each side, see different
boards, so one shared broadcast cannot work. A watch channel keeps only the latest change, so
a slow client skips stale boards instead of queueing them.

## Consequences

Whole boards are an estimated 20 KB at 200 units. Sending differences instead is left until
measurements call for it.
