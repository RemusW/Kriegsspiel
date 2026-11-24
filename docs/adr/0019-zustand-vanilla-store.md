# Client state lives in a Zustand vanilla store

Components read it through `useClientStore`; the engine is handed the store and uses
`getState` and `subscribe`. Per-frame data (camera, drag positions, presence) and
`PendingEdits` stay inside the engine. The engine is a plain class outside React, and state
such as the selection changes from both sides, so the store has to be usable outside React.
