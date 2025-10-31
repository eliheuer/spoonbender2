# Spoonbender

A font editor built with Xilem - a port of Runebender from Druid to Xilem and the Linebender ecosystem.

## Status

Very alpha and mostly Claude generated.

## Building

```bash
cargo build
cargo run
```

Requires Rust 1.88+ (uses 2024 edition features).

## Usage

```bash
# Run with GUI file picker
cargo run

# Open a UFO directly from command line
cargo run path/to/font.ufo

# Or after building
./target/release/spoonbender path/to/font.ufo
```

