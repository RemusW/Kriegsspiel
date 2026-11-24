# Terrain is delivered baked, and clients rebuild the geometry

Terrain is baked to a 16-bit heightmap and a colour texture, and each client rebuilds the
geometry from them. That is estimated to be 25 to 50 times smaller than sending a mesh; a full
map is estimated at 5 to 20 MB, cached by the client.
