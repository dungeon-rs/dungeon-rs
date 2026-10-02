//! The File menu's work: Open, Save, Save As, and Quit, the native file dialogs they run, the
//! save, discard, or cancel question asked while unsaved changes stand in the way, and the
//! report of what an opened Project is missing.
//!
//! Save and Open are requests to the project Manager; the Editor only chooses the files and
//! shows the answers. Unsaved changes are a history position that differs from the saved mark.

use crate::outcomes::counted;
use crate::panels::Outgoing;
use crate::state::{EditorState, Pending, Phase, Question};
use bevy::app::AppExit;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::query::With;
use bevy::ecs::system::{Query, Res, SystemParam};
use drs_history::History;
use drs_model::{
    Bounds, Element, ElementKindRegistry, Layer, Level, MissingAsset, MissingReason, OpenProject,
    OpenReport, PROJECT_EXTENSION, Project, Prop, Resolution, ResolutionTable, SaveProject,
    SavedMark, UnknownKind,
};
use std::path::PathBuf;

/// The Project as the File menu and the dialogs read it.
#[derive(SystemParam)]
pub(crate) struct ProjectView<'w, 's> {
    /// The Project's file and the history's position when it was last saved or opened.
    mark: Res<'w, SavedMark>,
    /// Where the history stands now.
    history: Res<'w, History>,
    /// The Project's Bounds and where its Asset References load from.
    projects: Query<'w, 's, (&'static Bounds, &'static ResolutionTable), With<Project>>,
    /// Every Level with its Layers.
    levels: Query<'w, 's, (Entity, &'static Level, Option<&'static Children>)>,
    /// Every Layer's Elements in stacking order.
    layers: Query<'w, 's, &'static Children, With<Layer>>,
    /// Every Element's common component and, for a Prop, the Asset it shows.
    elements: Query<'w, 's, (&'static Element, Option<&'static Prop>)>,
    /// The Element kinds this editor knows.
    kinds: Res<'w, ElementKindRegistry>,
}

impl ProjectView<'_, '_> {
    /// Whether the Project has unsaved changes: the history stands elsewhere than at the save.
    pub fn unsaved(&self) -> bool {
        self.mark.unsaved(&self.history)
    }

    /// Whether the Project has a file to save to without asking.
    pub fn has_file(&self) -> bool {
        self.mark.file.is_some()
    }

    /// The name the Author knows the Project by: its file's name, or `Untitled`.
    pub fn name(&self) -> String {
        self.mark.name()
    }

    /// The Level an Export is of: the Project's only one for now.
    pub fn level(&self) -> Option<(Entity, &Level)> {
        self.levels
            .iter()
            .next()
            .map(|(entity, level, _)| (entity, level))
    }

    /// The Bounds every Export covers.
    pub fn bounds(&self) -> Option<Bounds> {
        self.projects.iter().next().map(|(bounds, _)| *bounds)
    }

    /// How many Elements of a Level are drawn as placeholders: those whose Asset is Missing and
    /// those of a kind this editor does not know.
    pub fn placeholders(&self, level: Entity) -> usize {
        let resolutions = self.projects.iter().next().map(|(_, table)| table);
        let Ok((_, _, Some(layers))) = self.levels.get(level) else {
            return 0;
        };
        layers
            .iter()
            .filter_map(|&layer| self.layers.get(layer).ok())
            .flat_map(|elements| elements.iter())
            .filter_map(|&element| self.elements.get(element).ok())
            .filter(|(element, prop)| {
                let unknown = self.kinds.get(&element.kind).is_none();
                let missing = prop.is_some_and(|prop| {
                    matches!(
                        resolutions.and_then(|table| table.get(prop.asset)),
                        Some(Resolution::Missing(_))
                    )
                });
                unknown || missing
            })
            .count()
    }
}

/// Open…: asks about unsaved changes first, then lets the Author pick a Project file.
pub(crate) fn open(state: &mut EditorState, view: &ProjectView, outgoing: &mut Outgoing) {
    if view.unsaved() {
        state.question = Some(Question::asking(Pending::Open));
    } else {
        proceed(Pending::Open, outgoing);
    }
}

/// Quit: asks about unsaved changes first, then exits.
pub(crate) fn quit(state: &mut EditorState, view: &ProjectView, outgoing: &mut Outgoing) {
    if view.unsaved() {
        state.question = Some(Question::asking(Pending::Quit));
    } else {
        proceed(Pending::Quit, outgoing);
    }
}

/// Save: to the Project's file without asking, or as Save As when it has none. `false` when
/// the Author cancelled the dialog, so nothing was sent.
pub(crate) fn save(view: &ProjectView, outgoing: &mut Outgoing) -> bool {
    if view.has_file() {
        outgoing.save.write(SaveProject { path: None });
        true
    } else {
        save_as(view, outgoing)
    }
}

/// Save As…: the platform's save dialog proposing the Project's name, then Save to the file
/// chosen. `false` when the Author cancelled the dialog, so nothing was sent.
pub(crate) fn save_as(view: &ProjectView, outgoing: &mut Outgoing) -> bool {
    let proposed = format!("{}.{PROJECT_EXTENSION}", view.name());
    let Some(path) = choose_save_file("Save Project", "Project", PROJECT_EXTENSION, &proposed)
    else {
        return false;
    };
    outgoing.save.write(SaveProject { path: Some(path) });
    true
}

/// Carries a pending action out: Open runs the open dialog and sends Open for the file picked,
/// Quit exits.
fn proceed(pending: Pending, outgoing: &mut Outgoing) {
    match pending {
        Pending::Open => {
            if let Some(path) = choose_open_file() {
                outgoing.open.write(OpenProject { path });
            }
        }
        Pending::Quit => {
            outgoing.exit.write(AppExit::Success);
        }
    }
}

/// The save, discard, or cancel question, while one is asked; a question whose save has been
/// answered goes ahead with what the Author was about to do.
pub(crate) fn question(
    ctx: &egui::Context,
    state: &mut EditorState,
    view: &ProjectView,
    outgoing: &mut Outgoing,
) {
    let Some(question) = &mut state.question else {
        return;
    };
    if question.phase == Phase::Proceed {
        let pending = question.pending;
        state.question = None;
        proceed(pending, outgoing);
        return;
    }
    let action = match question.pending {
        Pending::Open => "opening another Project",
        Pending::Quit => "quitting",
    };
    let saving = question.phase == Phase::Saving;
    let modal = egui::Modal::new(egui::Id::new("unsaved-changes")).show(ctx, |ui| {
        ui.set_width(420.0);
        ui.heading("Unsaved changes");
        ui.label(format!(
            "The Project {} has unsaved changes. Save them before {action}?",
            view.name()
        ));
        if let Some(refusal) = &question.refusal {
            ui.colored_label(egui::Color32::LIGHT_RED, refusal);
        }
        if saving {
            ui.weak("Saving…");
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let save = ui.add_enabled(!saving, egui::Button::new("Save")).clicked();
            let discard = ui
                .add_enabled(!saving, egui::Button::new("Discard"))
                .clicked();
            let cancel = ui
                .add_enabled(!saving, egui::Button::new("Cancel"))
                .clicked();
            (save, discard, cancel)
        })
        .inner
    });
    let (save, discard, cancel) = modal.inner;
    if saving {
        return;
    }
    if cancel || modal.should_close() {
        state.question = None;
    } else if discard {
        question.phase = Phase::Proceed;
    } else if save {
        question.refusal = None;
        if self::save(view, outgoing) {
            question.phase = Phase::Saving;
        } else {
            state.question = None;
        }
    }
}

/// The report of what an opened Project is missing, until the Author dismisses it; the
/// placeholders stay.
pub(crate) fn report(ctx: &egui::Context, state: &mut EditorState) {
    let Some(report) = &state.report else {
        return;
    };
    let modal = egui::Modal::new(egui::Id::new("open-report")).show(ctx, |ui| {
        ui.set_width(560.0);
        ui.heading("Opened with placeholders");
        ui.label(
            "The Project opened, but some of it cannot be shown on this device. Each item \
             below is drawn as a placeholder of its recorded size until it can be.",
        );
        ui.add_space(8.0);
        egui::ScrollArea::vertical()
            .max_height(360.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                if !report.missing_assets.is_empty() {
                    ui.strong("Missing Assets");
                    for asset in &report.missing_assets {
                        ui.label(describe_missing(asset));
                    }
                }
                if !report.unknown_kinds.is_empty() {
                    ui.add_space(4.0);
                    ui.strong("Unknown Element kinds");
                    for kind in &report.unknown_kinds {
                        ui.label(describe_unknown(kind));
                    }
                }
            });
        ui.add_space(8.0);
        ui.button("Dismiss").clicked()
    });
    if modal.inner || modal.should_close() {
        state.report = None;
    }
}

/// What the Author is told about a Missing Asset, in plain terms.
fn describe_missing(asset: &MissingAsset) -> String {
    let uses = counted(asset.elements, "Element", "Elements");
    let reason = match &asset.reason {
        MissingReason::FolderAbsent => {
            "the Asset Folder is not added on this device; add it under that name".to_owned()
        }
        MissingReason::AssetAbsent { version } => {
            format!("the Asset Folder on this device is version {version} and holds no such Asset")
        }
        MissingReason::Ambiguous {
            version,
            candidates,
        } => format!(
            "the Asset Folder on this device is version {version} and {} differ from the \
             recorded place only in spelling ({}), so none was chosen",
            counted(candidates.len(), "file", "files"),
            candidates.join(", ")
        ),
    };
    format!(
        "• {} from the Asset Folder {} (recorded version {}), used by {uses}: {reason}.",
        asset.name, asset.folder, asset.recorded_version
    )
}

/// What the Author is told about an unknown Element kind.
fn describe_unknown(kind: &UnknownKind) -> String {
    format!(
        "• {} of the kind {}, which this editor does not know; they are kept and saved back as \
         they are.",
        counted(kind.elements, "Element", "Elements"),
        kind.kind.as_str()
    )
}

/// The report's one-line summary for the status line.
pub(crate) fn summarise(report: &OpenReport) -> String {
    let missing = report.missing_assets.len();
    let unknown = report.unknown_kinds.len();
    let assets = || counted(missing, "Missing Asset", "Missing Assets");
    let kinds = || counted(unknown, "unknown Element kind", "unknown Element kinds");
    match (missing, unknown) {
        (0, 0) => String::new(),
        (_, 0) => format!(" with {}", assets()),
        (0, _) => format!(" with {}", kinds()),
        (_, _) => format!(" with {} and {}", assets(), kinds()),
    }
}

/// The platform's open dialog filtered to Project files, or in development builds the file
/// `DRS_PICK_FILE` names when it is set, an empty value standing for a cancelled dialog.
fn choose_open_file() -> Option<PathBuf> {
    #[cfg(feature = "dev")]
    if let Some(path) = std::env::var_os("DRS_PICK_FILE") {
        return (!path.is_empty()).then(|| PathBuf::from(path));
    }
    rfd::FileDialog::new()
        .set_title("Open Project")
        .add_filter("Project", &[PROJECT_EXTENSION])
        .pick_file()
}

/// The platform's save dialog proposing `proposed` and filtered to `extension`, or in
/// development builds the file `DRS_SAVE_FILE` names when it is set, an empty value standing
/// for a cancelled dialog.
pub(crate) fn choose_save_file(
    title: &str,
    filter: &str,
    extension: &str,
    proposed: &str,
) -> Option<PathBuf> {
    #[cfg(feature = "dev")]
    if let Some(path) = std::env::var_os("DRS_SAVE_FILE") {
        return (!path.is_empty()).then(|| PathBuf::from(path));
    }
    rfd::FileDialog::new()
        .set_title(title)
        .add_filter(filter, &[extension])
        .set_file_name(proposed)
        .save_file()
}
