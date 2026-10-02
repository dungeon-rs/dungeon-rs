//! The Assets panel: a search field, a grid of thumbnails of the Assets of every added Asset
//! Folder or of those the search text matches, one click to choose.
//!
//! Only the rows in view are laid out. Their thumbnails are loaded through the `thumb://` asset
//! source, decoded off the main thread by the asset system, registered with egui while their row
//! is laid out, and kept in a bounded set of the most recently shown. The library Manager answers
//! the search text; the browser never matches names itself.

use crate::state::{Chosen, EditorState};
use bevy::asset::{AssetPath, AssetServer, Handle, LoadState, RenderAssetUsages};
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Query, Res, ResMut, SystemParam};
use bevy::image::{Image, ImageFormatSetting, ImageLoaderSettings};
use bevy::math::UVec2;
use bevy_egui::{EguiContexts, EguiTextureHandle};
use drs_model::{
    AssetAddress, AssetFolder, Browse, FolderKey, IndexedAsset, SearchMatches, THUMBNAIL_SOURCE,
    ThumbnailState, Thumbnails,
};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::Path;

/// The side of a cell's square, in points.
const CELL: f32 = 128.0;
/// How many rows either side of those laid out are asked for and loaded ahead.
const PREFETCH_ROWS: usize = 2;
/// How many decoded thumbnails are kept at most.
const KEPT: usize = 512;

/// What the browser keeps between frames: no domain state, only what it has drawn, loaded, and
/// sent.
#[derive(Resource, Default)]
pub(crate) struct Browser {
    /// The thumbnails loaded, by Asset.
    loaded: BTreeMap<AssetAddress, Loaded>,
    /// The frames drawn, for the order thumbnails were last shown in.
    frame: u64,
    /// The search text last sent to the library Manager.
    searched: String,
    /// The Assets last named to the library Manager as wanted.
    wanted: Vec<AssetAddress>,
    /// What the grid showed in the last frame: the answered text it showed the matches of, or
    /// `None` for the whole library, so that a new text starts at the top.
    shown: Option<String>,
    /// Whether the panel was drawn in this frame.
    drawn: bool,
    /// The cells laid out in the last frame, for the development script to describe.
    #[cfg(feature = "dev")]
    pub cells: Vec<Cell>,
    /// The line above the grid in the last frame: the count, or that nothing matches.
    #[cfg(feature = "dev")]
    pub summary: String,
}

#[cfg(feature = "dev")]
impl Browser {
    /// How many thumbnails are decoded and kept, and how many of them are registered with egui.
    pub(crate) fn textures(&self) -> (usize, usize) {
        let registered = self
            .loaded
            .values()
            .filter(|loaded| loaded.texture.is_some())
            .count();
        (self.loaded.len(), registered)
    }
}

/// One cell as it was laid out.
#[cfg(feature = "dev")]
pub(crate) struct Cell {
    /// The Asset's name.
    pub name: String,
    /// What the cell showed.
    pub shown: Shown,
    /// Where it was, in logical pixels.
    pub rect: egui::Rect,
}

/// What a cell shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shown {
    /// The square placeholder of a thumbnail not yet generated.
    Pending,
    /// The placeholder of a generated thumbnail that is not decoded yet.
    Loading,
    /// The thumbnail itself.
    Thumbnail,
    /// The broken placeholder.
    Broken,
}

/// A thumbnail loaded through the asset server.
struct Loaded {
    /// Holding it keeps the decoded image and its texture alive.
    handle: Handle<Image>,
    /// The texture egui draws it with, while its row is laid out.
    texture: Option<egui::TextureId>,
    /// The frame it was last shown or asked for.
    shown: u64,
}

/// What the browser reads and writes.
#[derive(SystemParam)]
pub(crate) struct Library<'w, 's> {
    /// What the browser keeps.
    browser: ResMut<'w, Browser>,
    /// The Asset Folders and their thumbnail states.
    folders: Query<'w, 's, (Entity, &'static AssetFolder, Option<&'static Thumbnails>)>,
    /// What the search text matches, as the library Manager last answered it.
    matches: Res<'w, SearchMatches>,
    /// Where thumbnails are loaded.
    assets: Res<'w, AssetServer>,
    /// Browse, to the library Manager.
    browse: MessageWriter<'w, Browse>,
}

/// The `thumb://` path of the Asset at `place` in the folder with `key`, assembled from its parts
/// so that a `#` or `?` in a file name stays part of the name.
fn thumbnail_path(key: &FolderKey, place: &str) -> AssetPath<'static> {
    AssetPath::from_path_buf(Path::new(key.as_str()).join(place)).with_source(THUMBNAIL_SOURCE)
}

/// Whether `text` holds a word, that is, anything but whitespace.
fn has_words(text: &str) -> bool {
    text.split_whitespace().next().is_some()
}

/// `count` with its thousands grouped by commas, as `400,000`.
fn grouped(count: usize) -> String {
    let digits = count.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

/// The Assets the grid shows, in its order: every Asset of every folder by Canonical Name and
/// then place, or the matches of a search text in the order the library Manager gave them.
enum Grid<'a> {
    /// Every Asset, folder by folder: each folder with the slot its first Asset takes.
    Library(Vec<(Entity, usize)>, usize),
    /// The matches of a text.
    Matches(&'a SearchMatches),
}

impl Grid<'_> {
    /// How many Assets the grid holds.
    fn len(&self) -> usize {
        match self {
            Self::Library(_, total) => *total,
            Self::Matches(matches) => matches.assets.len(),
        }
    }

    /// The Asset in a slot of the grid, as its folder and its position in the folder's index.
    fn get(&self, slot: usize) -> Option<(Entity, usize)> {
        match self {
            Self::Library(folders, total) => {
                if slot >= *total {
                    return None;
                }
                let folder = folders.partition_point(|(_, first)| *first <= slot) - 1;
                let (entity, first) = folders[folder];
                Some((entity, slot - first))
            }
            Self::Matches(matches) => matches
                .assets
                .get(slot)
                .map(|found| (found.folder, found.position as usize)),
        }
    }
}

/// The grid for this frame: the matches of the answered text while one is typed and answered,
/// or else every Asset of `folders`, which are in Canonical Name order.
fn grid_of<'a>(
    folders: &[(Entity, &AssetFolder)],
    matches: &'a SearchMatches,
    searching: bool,
) -> Grid<'a> {
    if searching {
        return Grid::Matches(matches);
    }
    let mut first = 0;
    let starts = folders
        .iter()
        .map(|(entity, folder)| {
            let start = (*entity, first);
            first += folder.assets.len();
            start
        })
        .collect();
    Grid::Library(starts, first)
}

/// The line above the grid: how many Assets the library holds, how many match, or that none
/// matches the `typed` text.
fn summary(grid: &Grid, typed: &str) -> String {
    match (grid, grid.len()) {
        (Grid::Library(..), 1) => "1 Asset".to_owned(),
        (Grid::Library(..), total) => format!("{} Assets", grouped(total)),
        (Grid::Matches(_), 0) => format!("No Asset matches \u{201c}{}\u{201d}", typed.trim()),
        (Grid::Matches(_), 1) => "1 Asset matches".to_owned(),
        (Grid::Matches(_), total) => format!("{} Assets match", grouped(total)),
    }
}

/// Draws the Assets panel.
pub(crate) fn show(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    library: &mut Library,
    contexts: &mut EguiContexts,
) {
    let typed = ui
        .horizontal(|ui| {
            ui.label("Search");
            ui.add(
                egui::TextEdit::singleline(&mut state.search)
                    .hint_text("part of a name, or folder/name")
                    .desired_width(f32::INFINITY),
            )
            .changed()
        })
        .inner;
    let Library {
        browser,
        folders,
        matches,
        assets,
        browse,
    } = library;
    let browser = &mut **browser;
    let matches: &SearchMatches = matches;
    browser.frame += 1;
    browser.drawn = true;
    #[cfg(feature = "dev")]
    browser.cells.clear();

    let mut sorted: Vec<(Entity, &AssetFolder)> = folders
        .iter()
        .map(|(entity, folder, _)| (entity, folder))
        .collect();
    sorted.sort_by(|a, b| a.1.name.cmp(&b.1.name).then(a.1.key.cmp(&b.1.key)));
    if sorted.is_empty() {
        ui.weak("No Asset Folder is added yet. Use Add Asset Folder… in the Library menu.");
        #[cfg(feature = "dev")]
        "no Asset Folder".clone_into(&mut browser.summary);
        release(browser, contexts, &BTreeSet::new());
        send(browser, browse, &state.search, Vec::new());
        return;
    }
    // Until the answer to the text typed arrives, a frame later, the last answer stays shown;
    // with nothing typed, the whole library is shown at once.
    let searching = has_words(&state.search) && matches.has_words;
    let grid = grid_of(&sorted, matches, searching);
    let shown = searching.then(|| matches.text.clone());
    let to_top = typed || shown != browser.shown;
    browser.shown = shown;

    let summary = summary(&grid, &state.search);
    ui.label(&summary);
    #[cfg(feature = "dev")]
    {
        browser.summary = summary;
    }
    folder_lines(ui, &sorted, searching.then_some(matches));
    ui.separator();

    let spacing = ui.spacing().item_spacing;
    let name_height = ui.text_style_height(&egui::TextStyle::Body);
    let width = ui.available_width() - ui.spacing().scroll.allocated_width();
    let columns = columns_in(width, spacing.x);
    let rows = grid.len().div_ceil(columns);
    let mut laid_out = BTreeSet::new();
    let mut range = 0..0;
    let mut area = egui::ScrollArea::vertical()
        .id_salt("thumbnail-grid")
        .auto_shrink(false);
    if to_top {
        area = area.vertical_scroll_offset(0.0);
    }
    area.show_rows(ui, CELL + name_height, rows, |ui, rows| {
        range = rows.clone();
        for row in rows {
            ui.horizontal(|ui| {
                for column in 0..columns {
                    let Some(slot) = grid.get(row * columns + column) else {
                        break;
                    };
                    let Some(asset) = asset_at(folders, slot) else {
                        continue;
                    };
                    laid_out.insert(asset.address.clone());
                    cell(ui, state, browser, assets, contexts, &asset, name_height);
                }
            });
        }
    });

    let ahead = range.start.saturating_sub(PREFETCH_ROWS)
        ..range.end.saturating_add(PREFETCH_ROWS).min(rows);
    prefetch(browser, &grid, folders, assets, &ahead, &range, columns);
    let wanted = wanted(&grid, folders, &ahead, columns);
    send(browser, browse, &state.search, wanted);
    release(browser, contexts, &laid_out);
}

/// Unregisters every thumbnail when the panel was not drawn in this frame, as when its tab is
/// behind another, so that no texture stays registered for a grid nobody sees.
pub(crate) fn release_if_hidden(library: &mut Library, contexts: &mut EguiContexts) {
    let browser = &mut *library.browser;
    if !std::mem::take(&mut browser.drawn) {
        #[cfg(feature = "dev")]
        browser.cells.clear();
        release(browser, contexts, &BTreeSet::new());
    }
}

/// How many cells fit side by side in `width`, at least one.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the count is floored and at least one, and a panel is far narrower than usize::MAX cells"
)]
fn columns_in(width: f32, spacing: f32) -> usize {
    ((width + spacing) / (CELL + spacing)).floor().max(1.0) as usize
}

/// One line per Asset Folder above the grid: its Canonical Name and how many of its Assets are
/// shown: all of them, or those the search text matches.
fn folder_lines(
    ui: &mut egui::Ui,
    folders: &[(Entity, &AssetFolder)],
    matches: Option<&SearchMatches>,
) {
    egui::ScrollArea::vertical()
        .id_salt("folder-lines")
        .max_height(ui.available_height() / 4.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for (entity, folder) in folders {
                let count = matches.map_or(folder.assets.len(), |matches| matches.count(*entity));
                ui.horizontal(|ui| {
                    ui.label(format!("{} ({})", folder.name, grouped(count)));
                    if folder.assets.is_empty() {
                        ui.weak("No Assets");
                    } else if count == 0 {
                        ui.weak("No Asset matches");
                    }
                });
            }
        });
}

/// An Asset of the grid with what the cell needs to draw it.
struct GridAsset<'a> {
    /// The folder it sits in.
    folder: &'a AssetFolder,
    /// The Asset.
    asset: &'a IndexedAsset,
    /// Where its thumbnail stands.
    state: ThumbnailState,
    /// How the browser names it.
    address: AssetAddress,
}

/// The Asset at `position` in the index of the folder on `entity`.
fn asset_at<'a>(
    folders: &'a Query<(Entity, &AssetFolder, Option<&Thumbnails>)>,
    (entity, position): (Entity, usize),
) -> Option<GridAsset<'a>> {
    let (_, folder, thumbnails) = folders.get(entity).ok()?;
    let asset = folder.assets.get(position)?;
    let state = thumbnails
        .and_then(|thumbnails| thumbnails.states.get(position).copied())
        .unwrap_or_default();
    Some(GridAsset {
        folder,
        asset,
        state,
        address: AssetAddress {
            folder: folder.key.clone(),
            place: asset.place.clone(),
        },
    })
}

/// The thumbnail of an Asset loaded through the asset server, loading it if it is not yet.
fn load<'a>(
    browser: &'a mut Browser,
    assets: &AssetServer,
    address: &AssetAddress,
) -> &'a mut Loaded {
    let frame = browser.frame;
    let loaded = browser.loaded.entry(address.clone()).or_insert_with(|| {
        let handle = assets
            .load_builder()
            .with_settings(|settings: &mut ImageLoaderSettings| {
                settings.format = ImageFormatSetting::Guess;
                settings.asset_usage = RenderAssetUsages::RENDER_WORLD;
            })
            .load(thumbnail_path(&address.folder, &address.place));
        Loaded {
            handle,
            texture: None,
            shown: frame,
        }
    });
    loaded.shown = frame;
    loaded
}

/// One cell: the thumbnail or a placeholder in a square, the name beneath, the full name, folder,
/// and place on hover, and a click to choose the Asset.
fn cell(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    browser: &mut Browser,
    assets: &AssetServer,
    contexts: &mut EguiContexts,
    asset: &GridAsset,
    name_height: f32,
) {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(CELL, CELL + name_height), egui::Sense::click());
    let square = egui::Rect::from_min_size(rect.min, egui::vec2(CELL, CELL));
    let visuals = ui.visuals().clone();
    let painter = ui.painter_at(rect);
    let chosen = state
        .chosen
        .as_ref()
        .is_some_and(|chosen| chosen.asset == asset.address);
    if chosen {
        painter.rect_filled(rect, 4.0, visuals.selection.bg_fill);
    } else if response.hovered() {
        painter.rect_filled(rect, 4.0, visuals.widgets.hovered.weak_bg_fill);
    }
    let placeholder = visuals.widgets.noninteractive.bg_fill;
    let shown = match asset.state {
        ThumbnailState::Pending => {
            painter.rect_filled(square.shrink(4.0), 4.0, placeholder);
            Shown::Pending
        }
        ThumbnailState::Broken => {
            broken(&painter, square, &visuals);
            Shown::Broken
        }
        ThumbnailState::Ready(size) => {
            let fitted = egui::Rect::from_center_size(
                square.center(),
                fitted(size, ui.ctx().pixels_per_point()),
            );
            let loaded = load(browser, assets, &asset.address);
            match assets.get_load_state(loaded.handle.id()) {
                Some(LoadState::Loaded) => {
                    let texture = *loaded.texture.get_or_insert_with(|| {
                        contexts.add_image(EguiTextureHandle::Strong(loaded.handle.clone()))
                    });
                    egui::Image::from_texture(egui::load::SizedTexture::new(
                        texture,
                        fitted.size(),
                    ))
                    .paint_at(ui, fitted);
                    Shown::Thumbnail
                }
                Some(LoadState::Failed(_)) => {
                    broken(&painter, square, &visuals);
                    Shown::Broken
                }
                Some(LoadState::NotLoaded | LoadState::Loading) | None => {
                    painter.rect_filled(fitted, 2.0, placeholder);
                    Shown::Loading
                }
            }
        }
    };
    let mut job = egui::text::LayoutJob::single_section(
        asset.asset.name.clone(),
        egui::TextFormat::simple(
            egui::TextStyle::Body.resolve(ui.style()),
            visuals.text_color(),
        ),
    );
    job.wrap = egui::text::TextWrapping::truncate_at_width(CELL);
    let galley = painter.layout_job(job);
    let name_at = egui::pos2(rect.center().x - galley.size().x / 2.0, square.bottom());
    painter.galley(name_at, galley, visuals.text_color());
    #[cfg(feature = "dev")]
    browser.cells.push(Cell {
        name: asset.asset.name.clone(),
        shown,
        rect,
    });
    // Only the development script reads what a cell showed.
    #[cfg(not(feature = "dev"))]
    let _ = shown;
    let response = response.on_hover_text(format!(
        "{}\n{}\n{}",
        asset.asset.name, asset.folder.name, asset.asset.place
    ));
    if response.clicked() {
        state.chosen = Some(Chosen {
            asset: asset.address.clone(),
            name: asset.asset.name.clone(),
        });
        state.selected = None;
    }
}

/// The size in points a thumbnail of `size` pixels is drawn at: one physical pixel of the
/// display per pixel of the thumbnail, so that it is never enlarged however many pixels a point
/// has, and scaled down, keeping its proportions, only when that would not fit in a cell.
fn fitted(size: UVec2, pixels_per_point: f32) -> egui::Vec2 {
    let pixels = |side: u32| f32::from(u16::try_from(side).unwrap_or(u16::MAX));
    let size = egui::vec2(pixels(size.x), pixels(size.y));
    let points_per_pixel = (1.0 / pixels_per_point).min(CELL / size.max_elem().max(1.0));
    size * points_per_pixel
}

/// The broken placeholder: a square crossed out.
fn broken(painter: &egui::Painter, square: egui::Rect, visuals: &egui::Visuals) {
    let inner = square.shrink(4.0);
    painter.rect_filled(inner, 4.0, visuals.extreme_bg_color);
    let stroke = egui::Stroke::new(2.0, visuals.error_fg_color);
    let cross = inner.shrink(inner.width() / 4.0);
    painter.line_segment([cross.left_top(), cross.right_bottom()], stroke);
    painter.line_segment([cross.right_top(), cross.left_bottom()], stroke);
}

/// Loads the thumbnails of the generated Assets in the rows either side of those laid out, so
/// they are decoded before they scroll into view; nothing is registered for them.
fn prefetch(
    browser: &mut Browser,
    grid: &Grid,
    folders: &Query<(Entity, &AssetFolder, Option<&Thumbnails>)>,
    assets: &AssetServer,
    ahead: &Range<usize>,
    laid_out: &Range<usize>,
    columns: usize,
) {
    let mut ready = Vec::new();
    for row in ahead.clone().filter(|row| !laid_out.contains(row)) {
        for column in 0..columns {
            let Some(slot) = grid.get(row * columns + column) else {
                break;
            };
            if let Some(asset) = asset_at(folders, slot)
                && matches!(asset.state, ThumbnailState::Ready(_))
            {
                ready.push(asset.address);
            }
        }
    }
    for address in ready {
        load(browser, assets, &address);
    }
}

/// The Assets of the rows laid out and those either side, whose thumbnails are wanted first.
fn wanted(
    grid: &Grid,
    folders: &Query<(Entity, &AssetFolder, Option<&Thumbnails>)>,
    ahead: &Range<usize>,
    columns: usize,
) -> Vec<AssetAddress> {
    (ahead.start * columns..ahead.end * columns)
        .map_while(|slot| grid.get(slot))
        .filter_map(|slot| asset_at(folders, slot).map(|asset| asset.address))
        .collect()
}

/// Sends Browse with the text typed and the Assets wanted, when either differs from what was
/// last sent, an empty text included.
fn send(
    browser: &mut Browser,
    browse: &mut MessageWriter<Browse>,
    typed: &str,
    wanted: Vec<AssetAddress>,
) {
    if typed != browser.searched || wanted != browser.wanted {
        typed.clone_into(&mut browser.searched);
        browser.wanted.clone_from(&wanted);
        browse.write(Browse {
            search: typed.to_owned(),
            wanted,
        });
    }
}

/// Unregisters the thumbnails whose rows are no longer laid out, and drops the least recently
/// shown beyond those kept, never one that is laid out.
fn release(browser: &mut Browser, contexts: &mut EguiContexts, laid_out: &BTreeSet<AssetAddress>) {
    for (address, loaded) in &mut browser.loaded {
        if loaded.texture.is_some() && !laid_out.contains(address) {
            contexts.remove_image(loaded.handle.id());
            loaded.texture = None;
        }
    }
    let excess = browser.loaded.len().saturating_sub(KEPT);
    if excess == 0 {
        return;
    }
    let mut oldest: Vec<(u64, AssetAddress)> = browser
        .loaded
        .iter()
        .filter(|(address, _)| !laid_out.contains(*address))
        .map(|(address, loaded)| (loaded.shown, address.clone()))
        .collect();
    oldest.sort();
    for (_, address) in oldest.into_iter().take(excess) {
        browser.loaded.remove(&address);
    }
}
