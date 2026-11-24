# A plain three.js engine class, not React Three Fiber

The scene is a plain three.js class that owns the canvas, mounted by one React component. The
owner knows three.js; dragging, the camera and hit testing are per-frame work that bypasses
React anyway; and the engine only needs the board pushed to it.
