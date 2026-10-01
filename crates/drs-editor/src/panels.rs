//! The window's layout: the menu bar, the docked Assets panel and viewport, the status line, and
//! the prompts and dialogs over them.

use crate::diagnostics::Diagnostics;
use crate::files::ProjectView;
use crate::state::{EditorState, NamePrompt};
use crate::{bindings, browser, diagnostics, export, files};
use bevy::app::AppExit;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{NonSendMarker, Query, Res, ResMut, SystemParam};
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::math::Rect;
use bevy_egui::EguiContexts;
use drs_history::History;
use drs_model::{
    AddFolder, AssetFolder, CanonicalName, ExportLevel, OpenProject, Redo, SaveProject, Undo,
    Viewport,
};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};

/// The panels the window is split into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tab {
    /// The Assets of every added Asset Folder.
    Assets,
    /// The Level, drawn by the render Engine behind this transparent tab.
    Viewport,
}

/// Where the panels sit.
#[derive(Resource)]
pub(crate) struct Layout(DockState<Tab>);

impl Default for Layout {
    /// The viewport with the Assets panel to its left.
    fn default() -> Self {
        let mut dock = DockState::new(vec![Tab::Viewport]);
        dock.main_surface_mut()
            .split_left(NodeIndex::root(), 0.25, vec![Tab::Assets]);
        Self(dock)
    }
}

/// The messages the panels send.
#[derive(SystemParam)]
pub(crate) struct Outgoing<'w> {
    /// Add Asset Folder, to the library Manager.
    add_folder: MessageWriter<'w, AddFolder>,
    /// Undo, to the authoring Manager.
    undo: MessageWriter<'w, Undo>,
    /// Redo, to the authoring Manager.
    redo: MessageWriter<'w, Redo>,
    /// Save and Save As, to the project Manager.
    pub save: MessageWriter<'w, SaveProject>,
    /// Open, to the project Manager.
    pub open: MessageWriter<'w, OpenProject>,
    /// Export Level, to the project Manager.
    pub export: MessageWriter<'w, ExportLevel>,
    /// Quit.
    pub exit: MessageWriter<'w, AppExit>,
}

/// Everything the panels read and write.
#[derive(SystemParam)]
pub(crate) struct Editor<'w, 's> {
    /// Where the panels sit.
    layout: ResMut<'w, Layout>,
    /// The Editor's own state.
    state: ResMut<'w, EditorState>,
    /// Where the Author is looking; the viewport tab lays its area out.
    viewport: ResMut<'w, Viewport>,
    /// The Asset Folders added on this device.
    folders: Query<'w, 's, &'static AssetFolder>,
    /// Whether there is anything to undo or redo.
    history: Res<'w, History>,
    /// Where the logs are, as the Host said.
    diagnostics: Res<'w, Diagnostics>,
    /// The Project as the File menu reads it.
    project: ProjectView<'w, 's>,
    /// The keys, for the menu's shortcuts.
    keys: Res<'w, ButtonInput<KeyCode>>,
    /// The messages to send.
    outgoing: Outgoing<'w>,
}

/// Draws the whole interface for one frame, or nothing while there is no primary egui context
/// to draw into yet.
///
/// Runs on the main thread because the file dialogs it may open are native dialogs.
pub(crate) fn draw(_main_thread: NonSendMarker, mut contexts: EguiContexts, mut editor: Editor) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let ctx = ctx.clone();
    shortcuts(&ctx, &mut editor);
    let mut root = egui::Ui::new(
        ctx.clone(),
        "root".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    menu_bar(&ctx, &mut root, &mut editor);
    status_line(&mut root, &editor.state);
    let Editor {
        layout,
        state,
        viewport,
        folders,
        ..
    } = &mut editor;
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(&mut root, |ui| {
            let mut panels = Panels {
                state,
                viewport,
                folders,
            };
            DockArea::new(&mut layout.0)
                .style(Style::from_egui(ui.style().as_ref()))
                .show_close_buttons(false)
                .show_leaf_close_all_buttons(false)
                .show_leaf_collapse_buttons(false)
                .show_inside(ui, &mut panels);
        });
    name_prompt(&ctx, &mut editor);
    let Editor {
        state,
        project,
        outgoing,
        ..
    } = &mut editor;
    files::question(&ctx, state, project, outgoing);
    files::report(&ctx, state);
    export::dialog(&ctx, state, project, outgoing);
}

/// What the File menu offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileAction {
    /// Open…
    Open,
    /// Save.
    Save,
    /// Save As…
    SaveAs,
    /// Export Level…
    Export,
    /// Quit.
    Quit,
}

/// Carries a File menu action out.
fn file_action(action: FileAction, editor: &mut Editor) {
    let Editor {
        state,
        project,
        outgoing,
        ..
    } = editor;
    match action {
        FileAction::Open => files::open(state, project, outgoing),
        FileAction::Save => {
            files::save(project, outgoing);
        }
        FileAction::SaveAs => {
            files::save_as(project, outgoing);
        }
        FileAction::Export => export::begin(state),
        FileAction::Quit => files::quit(state, project, outgoing),
    }
}

/// The File menu's shortcuts, while no dialog of the Editor's own is open and egui is not
/// using the keyboard.
fn shortcuts(ctx: &egui::Context, editor: &mut Editor) {
    if editor.state.modal_open() || ctx.egui_wants_keyboard_input() {
        return;
    }
    let keys = &editor.keys;
    let action = if bindings::any_pressed(bindings::OPEN, keys) {
        Some(FileAction::Open)
    } else if bindings::any_pressed(bindings::SAVE_AS, keys) {
        Some(FileAction::SaveAs)
    } else if bindings::any_pressed(bindings::SAVE, keys) {
        Some(FileAction::Save)
    } else if bindings::any_pressed(bindings::EXPORT, keys) && can_export(editor) {
        Some(FileAction::Export)
    } else if bindings::any_pressed(bindings::QUIT, keys) {
        Some(FileAction::Quit)
    } else {
        None
    };
    if let Some(action) = action {
        file_action(action, editor);
    }
}

/// Whether Export Level… is offered: there is a Level, and no Export is running.
fn can_export(editor: &Editor) -> bool {
    editor.project.level().is_some() && editor.state.exporting.is_none()
}

/// The menu bar: File, Library, Edit, and Help.
fn menu_bar(ctx: &egui::Context, root: &mut egui::Ui, editor: &mut Editor) {
    egui::Panel::top("menu").show(root, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            let mut chosen = None;
            ui.menu_button("File", |ui| {
                let entry =
                    |ui: &mut egui::Ui, label: &str, keys: &[bindings::Binding], enabled| {
                        let button = egui::Button::new(label)
                            .shortcut_text(bindings::shortcut_text(ctx, keys));
                        ui.add_enabled(enabled, button).clicked()
                    };
                if entry(ui, "Open…", bindings::OPEN, true) {
                    chosen = Some(FileAction::Open);
                }
                if entry(ui, "Save", bindings::SAVE, true) {
                    chosen = Some(FileAction::Save);
                }
                if entry(ui, "Save As…", bindings::SAVE_AS, true) {
                    chosen = Some(FileAction::SaveAs);
                }
                ui.separator();
                if entry(ui, "Export Level…", bindings::EXPORT, can_export(editor)) {
                    chosen = Some(FileAction::Export);
                }
                ui.separator();
                if entry(ui, "Quit", bindings::QUIT, true) {
                    chosen = Some(FileAction::Quit);
                }
            });
            if let Some(action) = chosen {
                file_action(action, editor);
            }
            ui.menu_button("Library", |ui| {
                if ui.button("Add Asset Folder…").clicked() {
                    pick_folder(&mut editor.state);
                }
            });
            // Neither is offered while a Prop is being dragged, as the drag is one step that is
            // still being recorded, nor while an Export runs, so the image is of one Level.
            let settled = !editor.state.dragging() && editor.state.exporting.is_none();
            ui.menu_button("Edit", |ui| {
                let undo = egui::Button::new("Undo")
                    .shortcut_text(bindings::shortcut_text(ctx, bindings::UNDO));
                if ui
                    .add_enabled(settled && editor.history.can_undo(), undo)
                    .clicked()
                {
                    editor.outgoing.undo.write(Undo);
                }
                let redo = egui::Button::new("Redo")
                    .shortcut_text(bindings::shortcut_text(ctx, bindings::REDO));
                if ui
                    .add_enabled(settled && editor.history.can_redo(), redo)
                    .clicked()
                {
                    editor.outgoing.redo.write(Redo);
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("Show Logs").clicked() {
                    diagnostics::show_logs(&mut editor.state, &editor.diagnostics);
                }
            });
        });
    });
}

/// Opens the platform's folder dialog and, when a folder is picked, the Canonical Name prompt
/// with the folder's own name proposed. Cancelling the dialog leaves nothing behind.
fn pick_folder(state: &mut EditorState) {
    let Some(path) = choose_folder() else {
        return;
    };
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    state.prompt = Some(NamePrompt {
        path,
        name,
        refusal: None,
        awaiting: false,
        focus: true,
    });
}

/// The platform's folder dialog, or in development builds the folder `DRS_PICK_FOLDER` names
/// when it is set, an empty value standing for a cancelled dialog.
fn choose_folder() -> Option<std::path::PathBuf> {
    #[cfg(feature = "dev")]
    if let Some(path) = std::env::var_os("DRS_PICK_FOLDER") {
        return (!path.is_empty()).then(|| std::path::PathBuf::from(path));
    }
    rfd::FileDialog::new()
        .set_title("Add Asset Folder")
        .pick_folder()
}

/// The status line: what happened last on the left, what the Author is doing on the right.
///
/// The right-hand hint is laid out first and the report is truncated to the width that is left,
/// so the two never overlap however long the report.
fn status_line(root: &mut egui::Ui, state: &EditorState) {
    egui::Panel::bottom("status").show(root, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if state.exporting.is_some() {
                ui.weak("The viewport waits for the Export");
            } else if let Some(chosen) = &state.chosen {
                ui.weak("Escape stops placing");
                ui.label(format!("placing {}", chosen.name));
            } else if state.selected.is_some() {
                ui.weak("Drag moves it, Delete removes it");
                ui.label("1 Prop selected");
            } else {
                ui.weak("Choose an Asset to place it, or click a Prop to select it");
            }
            ui.separator();
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add(egui::Label::new(&state.status).truncate());
            });
        });
    });
}

/// The Canonical Name prompt, while a folder is being added.
fn name_prompt(ctx: &egui::Context, editor: &mut Editor) {
    let Some(prompt) = &mut editor.state.prompt else {
        return;
    };
    let modal = egui::Modal::new(egui::Id::new("canonical-name")).show(ctx, |ui| {
        ui.set_width(420.0);
        ui.heading("Canonical Name");
        ui.label(format!("Folder: {}", prompt.path.display()));
        ui.label("The name Projects know this folder by, the same on every device.");
        ui.add_space(8.0);
        let field = ui.add_enabled(
            !prompt.awaiting,
            egui::TextEdit::singleline(&mut prompt.name)
                .hint_text("Canonical Name")
                .desired_width(f32::INFINITY),
        );
        if prompt.focus {
            field.request_focus();
            prompt.focus = false;
        }
        let entered = field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
        if let Some(refusal) = &prompt.refusal {
            ui.colored_label(egui::Color32::LIGHT_RED, refusal);
        }
        if prompt.awaiting {
            ui.weak("Adding…");
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let add = ui
                .add_enabled(!prompt.awaiting, egui::Button::new("Add"))
                .clicked();
            let cancel = ui.button("Cancel").clicked();
            (add || entered, cancel)
        })
        .inner
    });
    let (add, cancel) = modal.inner;
    if cancel || modal.should_close() {
        editor.state.prompt = None;
    } else if add && !prompt.awaiting {
        prompt.awaiting = true;
        prompt.refusal = None;
        editor.outgoing.add_folder.write(AddFolder {
            path: prompt.path.clone(),
            name: CanonicalName(prompt.name.clone()),
        });
    }
}

/// Draws the content of each tab.
struct Panels<'a, 'w, 's> {
    /// The Editor's own state.
    state: &'a mut EditorState,
    /// Where the Author is looking.
    viewport: &'a mut Viewport,
    /// The Asset Folders added on this device.
    folders: &'a Query<'w, 's, &'static AssetFolder>,
}

impl TabViewer for Panels<'_, '_, '_> {
    type Tab = Tab;

    fn id(&mut self, tab: &mut Tab) -> egui::Id {
        egui::Id::new(*tab)
    }

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        match tab {
            Tab::Assets => "Assets".into(),
            Tab::Viewport => "Viewport".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        match tab {
            Tab::Assets => browser::show(ui, self.state, self.folders),
            Tab::Viewport => {
                let rect = ui.available_rect_before_wrap();
                let area = Rect::new(rect.min.x, rect.min.y, rect.max.x, rect.max.y);
                if self.viewport.area != area {
                    self.viewport.area = area;
                }
            }
        }
    }

    fn is_closeable(&self, _tab: &Tab) -> bool {
        false
    }

    fn clear_background(&self, tab: &Tab) -> bool {
        *tab != Tab::Viewport
    }
}
