# Per-unit deltas rejected; presence is the real pressure

2026-11-23. The owner asked what the "per unit delta plan" in `CLAUDE.md` actually was, having
read it in a lesson rather than taking it at face value. It turned out to be two sentences of
intent with no design behind them, and examining it found a structural conflict: a delta is
only correct for a client holding every earlier one, which the watch channel's coalescing
deliberately does not guarantee. The line was struck from `CLAUDE.md` and replaced with the
reasoning, plus the cheaper alternative (serialize once per distinct view, not per
connection).

**Implications for teaching.** Two things change. First, do not teach deltas as a planned
future state of this system; teach snapshot self-healing as a property worth keeping, and
deltas as the trade-off that was declined. Second, step 5 (presence) is now the live
architectural question rather than a far-off one, since presence absorbs the problem deltas
were aimed at. Gambetta Part III (entity interpolation) becomes the relevant primary source,
and it meets the existing spring code in `pawns/spring.ts` &mdash; that is the natural bridge
from this network thread into territory the owner already knows well.

**Evidence of the reasoning floor this sets:** the question itself. Reading a planning
document and asking what is actually behind one of its lines, rather than implementing it, is
the behaviour the mission is aiming at. Pitch future lessons to a reader who will do that, and
keep flagging which statements in `CLAUDE.md` are measured and which are estimates.
