use propellant::{PropellantEngine, HasBuilder};



fn main() {
    // create a propellant engine instance, deletes it.
    let engine = PropellantEngine::builder();

    engine.main_loop().unwrap();
}