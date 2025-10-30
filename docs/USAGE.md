# Spoonbender Usage Guide

## Running the Application

```bash
# Development build
cargo run

# Release build (faster, recommended for actual use)
cargo run --release
```

## Loading a UFO Font

1. Launch Spoonbender
2. Click the "Open UFO..." button
3. Navigate to and select a `.ufo` directory
4. The font will be loaded and displayed

The application will show:
- Font name (Family + Style)
- Number of glyphs in the font
- Any error messages if loading fails

## Supported Formats

Spoonbender supports UFO (Unified Font Object) format version 3:
- `.ufo` directories containing glyph files
- Font metadata (fontinfo.plist)
- Glyph outlines (.glif files)
- Components and guides
- Kerning and groups

## Example UFO Fonts

You can test Spoonbender with publicly available UFO fonts:
- [Roboto](https://github.com/google/roboto) (contains UFO sources)
- [Source Sans Pro](https://github.com/adobe-fonts/source-sans-pro) (UFO format)
- [Open Sans](https://github.com/googlefonts/opensans) (may include UFO sources)

## Current Limitations

This is an early-stage project. Currently implemented:
- ✅ UFO file loading via file dialog
- ✅ Font metadata display
- ✅ Glyph counting

Not yet implemented:
- ❌ Glyph grid view
- ❌ Glyph editor
- ❌ Drawing tools
- ❌ Saving changes
- ❌ Creating new fonts

## Architecture

The application is built with:
- **Xilem 0.3** - Reactive UI framework
- **Norad 0.13** - UFO file format handling
- **rfd** - Native file dialogs

Key modules:
- `main.rs` - UI and application logic
- `data.rs` - Application state management
- `workspace.rs` - Font/UFO management with Norad
