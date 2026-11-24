# Kriegsspiel Planner

A tool for umpires to run Kriegsspiel games on a shared 3D table, with players seeing only
what their side may see.

## Language

### The table

**Board**:
Everything on the table at one moment: maps, units and features. The live board is the one
umpires are editing; each recorded turn keeps a board of its own.
_Avoid_: Scene, state, game

**Table**:
The surface the board is laid out on, extending past the maps so pieces can be set down
beside them.
_Avoid_: Ground, plane

**Map**:
An image laid on the table at a known ground scale. Several may be down at once, side by side
or overlapping.
_Avoid_: Sheet, map image, terrain

**Layer**:
A map's place in the stack: where two maps overlap, the one on the higher layer is seen.
_Avoid_: Order, z-order, depth

**Terrain**:
The shape of the ground: heights, beneath the maps.
_Avoid_: Map, heightmap

**Piece**:
Anything an umpire sets down on the maps and can pick up, turn and move: a unit, a feature or
a marker. Maps and line features are on the board but are not pieces.
_Avoid_: Pawn, token, entity, object

**Feature**:
A terrain object an umpire sets down at one spot, such as a house or a battery emplacement,
chosen from a list of kinds. It may be hidden from a side.
_Avoid_: Prop, model, building

**Marker**:
A piece showing a passing condition on the ground, such as smoke or fire, covering an area
around its spot. Markers come and go during play; features are terrain and stay put.
_Avoid_: Effect, particle, token

**Line feature**:
A terrain object that runs along a path, such as a fence, a barricade or a trench. It may be
hidden from a side.
_Avoid_: Obstacle, line, linear feature

**Ground scale**:
How much real ground a map shows per unit of its image. Every distance on the table is in
metres on the ground, so maps of different scales and the pieces on them all agree.
_Avoid_: Map scale (ambiguous with a 1:N ratio), world size

### Forces

**Unit**:
A military formation on the table, such as a battalion or a battery, belonging to one side.
_Avoid_: Pawn, block, counter, token

**Unit kind**:
The arm a unit belongs to: infantry, cavalry or artillery. A fixed list, extended as
scenarios need it.
_Avoid_: Unit type, class

**Formation**:
How a unit is drawn up, such as line, column or square for infantry, or limbered and
unlimbered for artillery. Together with the unit's kind it sets the ground the unit covers.
Each kind allows only some formations.
_Avoid_: Order, stance, mode

**Side**:
One of the opposing parties in a game; each player belongs to one. A scenario defines its
sides; until that exists, every game has Blue and Red. A unit always has a side; a feature,
marker or line feature may have none.
_Avoid_: Team, faction, player

### Time

**Event**:
Something that happened during play, such as cannon fire, shown when the turn is played back
but not a thing sitting on the table: it cannot be moved, selected or undone.
_Avoid_: Effect, action
