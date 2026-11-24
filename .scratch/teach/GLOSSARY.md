# Network model glossary

The language of the client-server model in this project. The repository's own
`GLOSSARY.md` at the root holds *domain* language (board, piece, umpire, turn); this file
holds *networking* language only. Keep the two from drifting.

A term earns a place here once it can be used correctly, not when it has been met once.

## Terms

**Authoritative server**:
The one copy of the game that counts. Clients send intentions and display what comes back;
they never hold state that could disagree.
_Avoid_: Host, master, source of truth

**Command**:
One message from a client asking for a change, stating the result it wants rather than the
input that produced it. Absolute, so sending it twice is harmless.
_Avoid_: Action, request, event, message

**Snapshot**:
The whole of what one client should show, in one message: a board, the turn list and which
turn that client is viewing. Taken under a single read of the document, so its parts cannot
disagree.
_Avoid_: Update, state, sync, payload

**Effect**:
The answer to "who has to be told?" after a command: nobody, the sender alone, or everyone.
_Avoid_: Result, status, scope

## Pending

Terms introduced in lesson 0001 but not yet promoted, because the owner has not yet used
them unprompted.

From lesson 0001: *polling*, *push*, *framing*, *event vs. resource*, *latency floor*,
*backpressure*.

From lesson 0002: *fan-out*, *coalescing*, *cancel safety*, *doorbell (vs. delivery)*.

From lesson 0003: *optimistic update* (never "prediction" in this project), *reconciliation*,
*pending edit*, *units as drawn*.

From lesson 0004: *origin*, *same-origin policy*, *identity continuity*.

From lesson 0007: *presence*, *roster*, *half-open connection*, *staleness window*,
*heartbeat*, *unsent prefix of an edit*. House rule: presence is never called an *edit* or a
*command* &mdash; it is a proposal, and the vocabulary should keep that straight, because the
whole design rests on it not entering `Doc`.

From lesson 0006: *entity interpolation*, *render in the past*, *dead reckoning*, *latest
target (vs. sample buffer)*, *settle time*. House rule set here: never call the springs
"interpolation" &mdash; they are a filter chasing a latest target, and conflating the two
hides the fact that one needs a queue and the other cannot have one.

From lesson 0005: *trigger / render / commit*, *reference identity*, *subscription (vs.
storage)*, *push vs. pull*, *per-frame state*. Note the house rule this lesson sets: say
"a store is a subscription", never "a store is where state lives" &mdash; the second is the
phrasing that makes the placement question unanswerable.
