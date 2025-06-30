# Project Structure

This document describes the improved folder structure of the Legend-of-PSU game project.

## Overview

The project has been reorganized into a more modular and maintainable structure that separates engine code from game-specific logic.

## Directory Structure

```
Legend-of-PSU/
├── assets/                    # Game assets (textures, fonts, etc.)
│   ├── dirt.png
│   ├── grass_block_side.png
│   ├── grass_block_top.png
│   ├── grass_block_top-2.png
│   ├── stone.png
│   └── dejavu-sans/          # Font files and license
├── Cargo.toml                # Rust package configuration
├── Cargo.lock                # Dependency lock file
└── src/
    ├── engine/               # Core engine code (reusable)
    │   ├── mod.rs           # Engine module declarations
    │   ├── graphics/        # Rendering, textures, vertex definitions
    │   │   ├── mod.rs
    │   │   ├── renderer.rs
    │   │   ├── texture.rs
    │   │   └── vertex.rs
    │   ├── input/           # Input handling
    │   │   └── mod.rs
    │   ├── shaders/         # Shader code
    │   │   ├── mod.rs
    │   │   └── shader.wgsl
    │   └── window/          # Window management
    │       └── mod.rs
    ├── game/                # Game-specific logic
    │   ├── mod.rs           # Game module declarations
    │   ├── player/          # Player-related code
    │   │   └── mod.rs
    │   ├── state/           # Game state management
    │   │   └── mod.rs
    │   └── world/           # World logic and game objects
    │       ├── mod.rs
    │       ├── app.rs       # Main application logic
    │       ├── camera.rs    # Camera system
    │       ├── chunk_manager.rs  # Chunk management
    │       └── chunk.rs     # Chunk data and generation
    ├── lib.rs               # Library entry point
    └── main.rs              # Application entry point
```

## Module Descriptions

### Engine Module (`src/engine/`)

Contains reusable core engine code that could be used across different games:

- **graphics/**: Rendering pipeline, texture management, vertex definitions
- **input/**: Input handling and event processing
- **shaders/**: GPU shader code and management
- **window/**: Window creation and management (placeholder for future implementation)

### Game Module (`src/game/`)

Contains game-specific logic and features:

- **player/**: Player movement, state, and interactions (placeholder for future implementation)
- **state/**: Game state management and transitions (placeholder for future implementation)
- **world/**: World generation, chunk management, camera system, and main application logic

## Benefits of This Structure

1. **Separation of Concerns**: Engine code is separated from game-specific logic
2. **Reusability**: Engine components can be reused in other projects
3. **Maintainability**: Clear organization makes the codebase easier to navigate and maintain
4. **Scalability**: Easy to add new features without cluttering existing modules
5. **Modularity**: Each module has a clear responsibility and API

## Usage

The main entry point is `src/main.rs`, which initializes the game and starts the main loop. The library API is exposed through `src/lib.rs` for potential use as a library.

## Future Enhancements

- Implement window management in `engine/window/`
- Add player functionality in `game/player/`
- Implement game state management in `game/state/`
- Add proper error handling and logging
- Consider adding a prelude module for commonly used types
