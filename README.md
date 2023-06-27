# Propellant

### Yet anoter Rust Game Engine.

Propellant is a Rust game engine developped over the Foundry ECS library.

## Philosophy

The idea of the Propellant game engine is to provide a simple and easy to use game engine games. It is not meant to be a full fledged game engine with an editor, but rather a library that provide basic tools for any game development, such as rendering, physics, audio, networking, etc.

The core concepts around which Propellant is built are:

- Only pay for what you use. The engine will not embark anything if you do not specify it.
- Rebuild what you don't like. The engine is built around the idea that you can replace any part of it with your own implementation, and tries it's best to make it easy to do so. 
- The engine is built over a ECS lib. If you want an entity, create it in the ECS. If you want a system, insert it in the ECS.

## State

The engine is currently in a very early stage of development. I am working on the Vulkan integration for rendering, and making a proper abstractions for it.

## Usage

The engine is built as a library. To install it, clone this repo and you can use it in your own project by adding it as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
propellant = { path = "path/to/propellant" }
```

Then, import the propellant engine library, create an engine instance and run it:

```rust
use propellant::*;

fn main() {
    let mut engine = PropellantEngine::default()
    engine.main_loop();
}
```

For more details about how to import resources, create entities and systems, have a look at the provided examples.

