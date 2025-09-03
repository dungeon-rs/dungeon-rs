//! A thin wrapper around the `egui-notify` crate to integrate it into Bevy's ECS.
//!
//! An additional benefit of this wrapper is we can switch to a different implementation later.

use bevy::prelude::Resource;
use egui::{Context, WidgetText};
use egui_notify::{Toast, Toasts};

/// Provides access to the notification system in the UI.
#[derive(Resource, Default)]
pub struct Notifications {
    /// The underlyling handle to the `egui-notify` crate.
    toasts: Toasts,
}

impl Notifications {
    /// Renders the notifications in the UI.
    pub fn ui(&mut self, ctx: &Context) {
        self.toasts.show(ctx);
    }

    /// Renders an *info* level notification.
    pub fn info(&mut self, text: impl Into<WidgetText>) {
        let toast = Toast::info(text);

        self.toasts.add(toast);
    }

    /// Renders an *warn* level notification.
    pub fn warn(&mut self, text: impl Into<WidgetText>) {
        let toast = Toast::warning(text);

        self.toasts.add(toast);
    }

    /// Renders an *error* level notification.
    pub fn error(&mut self, text: impl Into<WidgetText>) {
        let toast = Toast::error(text);

        self.toasts.add(toast);
    }
}
