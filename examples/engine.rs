use propellant::PropellantEngine;



fn main() {
    // create a propellant engine instance, deletes it.
    let engine = PropellantEngine::new();

    engine.main_loop();
}