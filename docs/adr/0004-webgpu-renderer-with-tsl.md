# WebGPURenderer with materials written in TSL

The client renders with `WebGPURenderer` from `three/webgpu`, and custom materials are written
in TSL, with raw WGSL through `wgslFn` where needed. It shows the current three.js pipeline,
and it falls back to WebGL2 by itself where WebGPU is missing, so one code path serves every
client.
