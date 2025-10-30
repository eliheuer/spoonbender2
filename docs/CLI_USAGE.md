# Command Line Interface

## Overview

Spoonbender supports opening UFO font files directly from the command line, similar to Runebender.

## Usage

### Basic GUI Mode

Launch Spoonbender with the file picker dialog:

```bash
cargo run
# or
./target/release/spoonbender
```

### Direct UFO Loading

Open a specific UFO file on startup:

```bash
cargo run path/to/font.ufo
# or
./target/release/spoonbender path/to/font.ufo
```

**Examples:**

```bash
# Absolute path
cargo run /home/user/fonts/MyFont.ufo

# Relative path
cargo run ../fonts/MyFont.ufo

# Current directory
cargo run ./MyFont.ufo
```

## Implementation

**File**: `src/main.rs` (lines 204-228)

The CLI argument handling:
1. Checks if any command-line arguments are provided
2. Takes the first argument as a file path
3. Validates that the path exists
4. Loads the UFO file before starting the UI
5. If the path is invalid, prints an error and shows usage

**Error Handling:**

If the provided path doesn't exist:
```
Error: Path does not exist: /path/to/nonexistent.ufo
Usage: spoonbender [path/to/font.ufo]
```

The application will still launch with the welcome screen.

## Features

- ✅ Accepts UFO directory paths as first argument
- ✅ Validates path existence before loading
- ✅ Shows helpful error messages for invalid paths
- ✅ Falls back to welcome screen if load fails
- ✅ Compatible with both relative and absolute paths

## Future Enhancements

Potential additions (not yet implemented):

- `-h` / `--help` flag for detailed usage
- `-v` / `--version` flag for version info
- `--new` flag to create a new font
- Multiple file support
- Verbose/debug output flags
