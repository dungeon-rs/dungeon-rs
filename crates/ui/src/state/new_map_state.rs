//! Contains the state of the new map form / window

use garde::Validate;

#[derive(Debug, Default, Validate)]
pub struct NewMapState {
    #[garde(ascii, length(min = 3, max = 100))]
    pub name: String,
    #[garde(range(min = 4, max = 10), custom(multiple_of_4))]
    pub width: u32,
    #[garde(range(min = 4, max = 10), custom(multiple_of_4))]
    pub height: u32,
}

fn multiple_of_4(value: &u32, context: &&&()) -> garde::Result {
    if value % 4 == 0 {
        return Ok(());
    }

    Err(garde::Error::new("Value must be a multiple of 4"))
}
