//! Registers a custom `panic!` handler that alerts the user of unrecoverable errors.
//!
//! The source of this module may read a little weird, but this is in fact an intentional workaround
//! of how the linters function. When running `cargo clippy` it will run *all* targets, including `test`.
//!
//! To get around this, when compiling under `test` we generate an empty function (hinted as inline
//! to further optimise), and for other targets we generate the full method. The `use` statements
//! are inlined in the method to prevent `unused import` warnings under `test` target.

#[cfg(test)]
#[inline]
pub fn register_panic_handler() {}

/// Registers a new `panic!` handler that alerts the user of unrecoverable errors.
#[cfg(not(test))]
#[allow(clippy::missing_panics_doc)]
pub fn register_panic_handler() {
    use bevy::prelude::error;
    use native_dialog::{DialogBuilder, MessageLevel};
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use sysinfo::System;

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let path = std::env::current_dir()
            .unwrap_or(PathBuf::from("."))
            .join("crash_report.txt");
        let message = if let Some(message) = info.payload().downcast_ref::<&'static str>() {
            String::from(*message)
        } else if let Some(message) = info.payload().downcast_ref::<String>() {
            message.clone()
        } else {
            String::from("An unrecoverable error has occurred.")
        };
        let location = if let Some(location) = info.location() {
            location.to_string()
        } else {
            String::from("Unknown location")
        };

        error!("An unrecoverable error has occurred: {:?}", info);
        DialogBuilder::message()
            .set_level(MessageLevel::Error)
            .set_title("Unrecoverable Error")
            .set_text(format!(
                "An unrecoverable error has occurred, the editor will shut down.
The error was: {message}

A crash file will be generated at {}",
                path.display()
            ))
            .alert()
            .show()
            .expect("Failed to show error dialog");

        let system = System::new_all();
        let os_version = System::long_os_version().unwrap_or(String::from("Unknown"));
        let mut dump_file = File::create(path).expect("Failed to create dump file");
        let backtrace = std::backtrace::Backtrace::force_capture();
        writeln!(dump_file, "--- DungeonRS Crash Report ---").unwrap();
        writeln!(
            dump_file,
            "Please provide the contents of this file when creating a bug report."
        )
        .unwrap();
        writeln!(dump_file).unwrap();
        writeln!(dump_file).unwrap();
        writeln!(dump_file, "Operating System: {os_version}").unwrap();
        writeln!(
            dump_file,
            "Memory: {}/{}",
            system.used_memory(),
            system.total_memory()
        )
        .unwrap();

        writeln!(
            dump_file,
            "CPU: {} Cores {}",
            system.cpus().len(),
            system.cpus()[0].brand()
        )
        .unwrap();
        for cpu in system.cpus() {
            writeln!(
                dump_file,
                "{}: {}Hz ({}) {}%",
                cpu.name(),
                cpu.frequency(),
                cpu.brand(),
                cpu.cpu_usage()
            )
            .unwrap();
        }

        writeln!(dump_file).unwrap();
        writeln!(dump_file, "---").unwrap();
        writeln!(dump_file, "Error: {message}").unwrap();
        writeln!(dump_file, "Location: {location}").unwrap();
        writeln!(dump_file, "Backtrace: {backtrace}").unwrap();
        writeln!(dump_file, "---").unwrap();
        writeln!(dump_file, "Raw: {info:?}").unwrap();

        dump_file.sync_all().unwrap();
        default_hook(info);
    }));
}
