use propellant::*;

fn main() {
    let engine = PropellantEngine::default()
        .with_window().unwrap();    

    engine.main_loop();
}
