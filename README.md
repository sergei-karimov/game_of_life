# Game of Life

A fast Conway's Game of Life simulation written in Rust with Macroquad.

## Features

- Parallelized generation updates using Rayon.
- Large simulation world with a `1_600 × 900` window.
- Camera zooming and panning.
- Rendering limited to the visible window for better performance.

## Controls

- Mouse wheel: zoom in and out around the cursor.
- Hold the left mouse button and drag: move around the world.

The camera is constrained to the world boundaries, so empty white borders cannot be exposed. Zooming out stops when the whole world fits the window.

## Run

Install Rust, then run the optimized build:

```bash
cargo run --release
```

Run the tests with:

```bash
cargo test
```

## Configuration

The main dimensions are defined in `src/main.rs`:

```rust
const WINDOW_WIDTH: u32 = 1_600;
const WINDOW_HEIGHT: u32 = 900;
const WORLD_WIDTH: usize = 8_000;
const WORLD_HEIGHT: usize = 4_500;
```

Larger worlds require considerably more memory because the simulation keeps two cell buffers in memory.
