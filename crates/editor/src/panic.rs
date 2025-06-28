//! Registers a custom `panic!` handler that alerts the user of unrecoverable errors.

use bevy::prelude::error;
use rfd::{MessageButtons, MessageDialog};

/// Registers a new `panic!` handler that alerts the user of unrecoverable errors.
pub fn register_panic_handler() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = if let Some(message) = info.payload().downcast_ref::<&'static str>() {
            String::from(*message)
        } else {
            String::from("An unrecoverable error has occurred.")
        };
        let location = if let Some(location) = info.location() {
            location.to_string()
        } else {
            String::from("Unknown location")
        };

        MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Unrecoverable Error")
            .set_buttons(MessageButtons::Ok)
            .set_description(format!(
                "An unrecoverable error has occured, the editor will shut down.
The error was: {message}

Error occurred at: {location}"
            ))
            .show();

        error!("An unrecoverable error has occurred: {:?}", info);
        default_hook(info);
    }));
}
