use foundry::Entity;


/// let's implement this the bad way first?
/// This does not get renderered, but creates chars that do.
/// This is super wanky at the moment, we need a system to layout the text.
pub struct UiTextRenderer {
    text: String,
    font: u32,
    characters: Vec<Entity>,
}




