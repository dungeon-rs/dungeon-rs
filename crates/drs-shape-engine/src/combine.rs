//! `CombineOutlines`: the outlines of a Layer combined in stacking order into the floor they
//! cover together, the Walls that run where that floor ends and between Rooms that meet edge to
//! edge, each Room's own floor, and which Rooms form one combination.
//!
//! Every closed outline is flattened as the Engine flattens it, each vertex on the exact curve,
//! and the outlines are folded in stacking order through `i_overlay`'s `EdgeOverlay` behind an
//! integer adapter shared by every pass, so a vertex has the same integer coordinates in each
//! pass and coincident edges stay exactly coincident. Each edge carries what it is a piece of: a
//! part of an outline and the range of parameters along it, through every split and merge, so
//! every edge of the result names the parts of the outlines it lies on. Coincident edges whose
//! outlines lie on opposite sides are the edges Rooms share; they leave the result's boundary,
//! and are kept from the records of the merges, clipped to the combined floor when a later cut
//! takes floor away beside them.
//!
//! The overlay works in integers and the rest in double precision, with square roots as the only
//! operation beyond arithmetic, so the result is the same on every machine.

use crate::path::{Curve, Path, flatten, parts_flattened};
use crate::room::{fill, fill_contours};
use crate::wall::vector;
use bevy_math::Vec2;
use drs_model::{FillMesh, LinePlace, LinePoint, Stretch};
use i_overlay::core::edge_data::{EdgeDataMerge, EdgeDataSplit, OverlayEdgeData};
use i_overlay::core::edge_overlay::{EdgeOverlay, InputEdge};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ShapeType;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::clip::FloatClip;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::segm::boolean::ShapeCountBoolean;
use i_overlay::string::clip::ClipRule;
use i_overlay::vector::edge::{CLIP_LEFT, CLIP_RIGHT, SUBJ_LEFT, SUBJ_RIGHT};
use kurbo::Point;
use std::collections::BTreeMap;

/// Closer than this, in Grid cells, two points of a Wall line count as one when Walls between
/// Rooms are joined end to end and clipped pieces are matched to the Wall they came from.
const SAME_POINT: f64 = 1e-6;

/// Shorter than this share of an edge's parameter range, a piece of a Wall between Rooms left by
/// clipping is no piece.
const NO_PIECE: f64 = 1e-9;

/// One outline of a Layer as `CombineOutlines` takes it: a Room's closed outline, which combines
/// with the Layer's other closed outlines and may cut, or a Wall's open line, which combines with
/// nothing.
#[derive(Debug, Clone, PartialEq)]
pub struct Outline {
    /// Its points and parts.
    pub path: Path,
    /// Whether it takes floor away from the outlines before it rather than adding its own.
    pub cuts: bool,
}

/// A Wall drawn in an outline's look: a line along pieces of its parts, each point tagged with
/// the part it lies on and the parameter along it. Where the line passes from one part to the
/// next, the point is given twice, once for each part, so every chord lies on one part.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WallLine {
    /// The points, in order; a closed line ends at its first point again.
    pub line: Vec<LinePoint>,
    /// Whether the line runs round, joined at its first point with no ends.
    pub closed: bool,
}

/// What `CombineOutlines` derives for one outline.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CombinedOutline {
    /// The whole outline flattened into its line, from the first point to the last, or round to
    /// the first again on a closed outline, each point tagged with its part and the parameter
    /// along it, whether or not a Wall runs along it.
    pub line: Vec<LinePoint>,
    /// Whether the line closes from its last point back to its first.
    pub closed: bool,
    /// How many parts the outline has.
    pub parts: usize,
    /// The floor: for a closed outline that does not cut, everything its line winds around under
    /// the non-zero rule, less what the closed outlines after it that cut take away; nothing for
    /// an open line or an outline that cuts.
    pub floor: FillMesh,
    /// The Walls drawn in its look: an open line's whole line, or the pieces of a closed
    /// outline's edges along which the combined floor ends or which it shares with another
    /// outline, where it is the last of the outlines whose edges lie there.
    pub walls: Vec<WallLine>,
    /// The places on its parts where a Wall runs, whichever outline's look it is drawn in, each
    /// a range of parameters on one part.
    pub walled: Vec<Stretch>,
    /// The number of the last outline, in stacking order, of the combination it belongs to: the
    /// outlines whose floors overlap or touch, directly or through others, cuts included. An open
    /// line is a combination of its own.
    pub last: usize,
}

/// What `CombineOutlines` derives for the outlines of a Layer: each outline's own part, in the
/// order they were given, and the Walls of the whole Layer that Portals are measured along.
#[derive(Debug, Clone, Default)]
pub struct Combination {
    /// Each outline's own part, in the order given.
    pub outlines: Vec<CombinedOutline>,
    /// The outlines as given, which Portals stand on.
    pub(crate) paths: Vec<Path>,
    /// The Walls, each a line of chords that may run along several outlines' parts.
    pub(crate) chains: Vec<Chain>,
}

/// A piece of one part of an outline: the parameters along that part at the two ends of a chord.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Source {
    /// The outline, by its number.
    pub(crate) outline: usize,
    /// The part.
    pub(crate) part: usize,
    /// The parameter at the chord's start.
    pub(crate) from: f64,
    /// The parameter at the chord's end.
    pub(crate) to: f64,
}

impl Source {
    /// The parameter at a share of the chord from its start.
    pub(crate) fn at(&self, share: f64) -> f64 {
        self.from + (self.to - self.from) * share
    }
}

/// A chord of a Wall: where it runs, the outline whose look it is drawn in, and every outline
/// whose part lies along it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Chord {
    /// Its start.
    pub(crate) a: Point,
    /// Its end.
    pub(crate) b: Point,
    /// The piece of the outline it is drawn for: the last of its sources.
    pub(crate) owner: Source,
    /// The pieces of every outline lying along it, the owner's included.
    pub(crate) sources: Vec<Source>,
}

/// A Wall of the Layer: chords end to end, round the edge of the combined floor or along an edge
/// two Rooms share.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Chain {
    /// Whether the last chord ends where the first starts.
    pub(crate) closed: bool,
    /// The chords, in order.
    pub(crate) chords: Vec<Chord>,
}

/// `CombineOutlines`: the outlines of a Layer in stacking order, combined.
///
/// The closed outlines are folded in their order: each run of outlines that do not cut adds
/// what they wind around to the combined floor, and each run of outlines that cut takes what
/// they wind around away from it, so an outline after a cut adds floor as any other. A Wall runs
/// along every stretch of an outline's edge with the combined floor on one side and not on the
/// other, and along every stretch where edges of two outlines that do not cut lie exactly on one
/// another with what they wind around on opposite sides, wherever the combined floor lies on
/// both sides of it. Each stretch of Wall is drawn in the look of the last outline whose edge lies
/// along it. Each outline that does not cut keeps its own floor, less what the outlines after it
/// that cut take away.
///
/// An open line combines with nothing: its Wall is its whole line, and it has no floor.
#[must_use]
pub fn combine_outlines(outlines: &[Outline]) -> Combination {
    let lines: Vec<Vec<LinePoint>> = outlines
        .iter()
        .map(|outline| flatten(&outline.path))
        .collect();
    let mut combined: Vec<CombinedOutline> = outlines
        .iter()
        .zip(&lines)
        .enumerate()
        .map(|(index, (outline, line))| CombinedOutline {
            line: line.clone(),
            closed: outline.path.closed,
            parts: outline.path.parts(),
            last: index,
            ..CombinedOutline::default()
        })
        .collect();
    let mut chains = Vec::new();
    for (index, outline) in outlines.iter().enumerate() {
        if !outline.path.closed {
            let chain = open_chain(index, &outline.path);
            combined[index].walled = whole(&outline.path);
            combined[index].walls = vec![WallLine {
                line: lines[index].clone(),
                closed: false,
            }];
            chains.push(chain);
        }
    }
    let rooms: Vec<usize> = (0..outlines.len())
        .filter(|index| outlines[*index].path.closed)
        .collect();
    if !rooms.is_empty() {
        let mut fold = Fold::new(outlines, &rooms);
        fold.run();
        fold.floors(&lines, &mut combined);
        fold.combinations(&mut combined);
        let (room_chains, shared) = fold.walls();
        let paths: Vec<&Path> = outlines.iter().map(|outline| &outline.path).collect();
        for chain in room_chains.iter().chain(&shared) {
            for (owner, line) in wall_lines(&paths, chain) {
                combined[owner].walls.push(line);
            }
        }
        let mut walled: BTreeMap<(usize, usize), Vec<(f64, f64)>> = BTreeMap::new();
        for chord in room_chains
            .iter()
            .chain(&shared)
            .flat_map(|chain| &chain.chords)
        {
            for source in &chord.sources {
                walled
                    .entry((source.outline, source.part))
                    .or_default()
                    .push((source.from.min(source.to), source.from.max(source.to)));
            }
        }
        for ((outline, part), ranges) in walled {
            combined[outline].walled.extend(
                merged(ranges)
                    .into_iter()
                    .map(|(from, to)| range(part, from, to)),
            );
        }
        chains.extend(room_chains);
        chains.extend(shared);
    }
    Combination {
        outlines: combined,
        paths: outlines
            .iter()
            .map(|outline| outline.path.clone())
            .collect(),
        chains,
    }
}

/// The places of every part of `path`, whole.
fn whole(path: &Path) -> Vec<Stretch> {
    (0..path.parts())
        .map(|part| range(part, 0.0, 1.0))
        .collect()
}

/// The stretch of part `part` from parameter `from` to `to`, `from` not above `to`.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the model keeps parameters in single precision"
)]
fn range(part: usize, from: f64, to: f64) -> Stretch {
    Stretch {
        start: LinePlace {
            segment: part,
            t: from as f32,
        },
        end: LinePlace {
            segment: part,
            t: to as f32,
        },
    }
}

/// Ranges sorted and joined where they overlap or meet.
fn merged(mut ranges: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    ranges.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let mut joined: Vec<(f64, f64)> = Vec::with_capacity(ranges.len());
    for (from, to) in ranges {
        match joined.last_mut() {
            Some(last) if from <= last.1 => last.1 = last.1.max(to),
            _ => joined.push((from, to)),
        }
    }
    joined
}

/// The Wall of an open line: its chords end to end, each its own.
fn open_chain(index: usize, path: &Path) -> Chain {
    let mut chords = Vec::new();
    for (part, points) in parts_flattened(path).into_iter().enumerate() {
        for pair in points.windows(2) {
            let source = Source {
                outline: index,
                part,
                from: pair[0].0,
                to: pair[1].0,
            };
            chords.push(Chord {
                a: pair[0].1,
                b: pair[1].1,
                owner: source,
                sources: vec![source],
            });
        }
    }
    Chain {
        closed: false,
        chords,
    }
}

/// A piece of a part of an outline as an edge of the overlay carries it: the parameters along
/// the part at the two ends of the edge's base segment, and whether what the outline adds lies
/// to the left of that segment.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Piece {
    /// The outline, by its number.
    outline: usize,
    /// The part.
    part: usize,
    /// The parameter at the base segment's start.
    t0: f64,
    /// The parameter at the base segment's end.
    t1: f64,
    /// Whether the floor this piece bounds lies to the left of the base segment.
    left: bool,
    /// Whether the outline cuts.
    cuts: bool,
}

impl Piece {
    /// The piece between two shares of its base segment, `from` at the new start and `to` at
    /// the new end, its side turned when the two run the other way.
    fn between(&self, from: f64, to: f64) -> Self {
        Self {
            t0: self.t0 + (self.t1 - self.t0) * from,
            t1: self.t0 + (self.t1 - self.t0) * to,
            left: if to >= from { self.left } else { !self.left },
            ..*self
        }
    }

    /// The piece as a chord's source.
    fn source(&self) -> Source {
        Source {
            outline: self.outline,
            part: self.part,
            from: self.t0,
            to: self.t1,
        }
    }
}

/// The data an overlay edge carries: a group of pieces lying along one base segment, and the
/// shares of that segment at the edge's start and end, so splitting and reversing an edge needs
/// no new group.
#[derive(Debug, Clone, Copy, PartialEq)]
struct EdgeData {
    /// The group, by its number in the store.
    group: usize,
    /// The share of the base segment at the edge's start.
    from: f64,
    /// The share of the base segment at the edge's end.
    to: f64,
}

/// Every group of pieces, and what the pass under way merged.
#[derive(Debug, Default)]
struct Store {
    /// The groups, by number.
    groups: Vec<Vec<Piece>>,
    /// The groups the pass under way made by merging coincident edges.
    merged: Vec<usize>,
    /// The ranges of shares of each group that a later merge of the pass under way took up.
    consumed: BTreeMap<usize, Vec<(f64, f64)>>,
}

impl Store {
    /// A new group of `pieces`, as the data of an edge along its whole base segment.
    fn group(&mut self, pieces: Vec<Piece>) -> EdgeData {
        self.groups.push(pieces);
        EdgeData {
            group: self.groups.len() - 1,
            from: 0.0,
            to: 1.0,
        }
    }

    /// The pieces an edge carries, with the edge's start and end as their base segment.
    fn pieces(&self, data: EdgeData) -> Vec<Piece> {
        self.groups
            .get(data.group)
            .map(|pieces| {
                pieces
                    .iter()
                    .map(|piece| piece.between(data.from, data.to))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// The share of the way from `a` to `b` at which `p` lies.
fn split_share<I: IntNumber>(context: &EdgeDataSplit<I>) -> f64 {
    let (ax, ay) = (context.a.x.to_f64(), context.a.y.to_f64());
    let (dx, dy) = (context.b.x.to_f64() - ax, context.b.y.to_f64() - ay);
    let (px, py) = (context.p.x.to_f64() - ax, context.p.y.to_f64() - ay);
    let squared = dx * dx + dy * dy;
    if squared > 0.0 {
        ((px * dx + py * dy) / squared).clamp(0.0, 1.0)
    } else {
        0.5
    }
}

impl OverlayEdgeData for EdgeData {
    type Store = Store;

    fn reversed(self, _: &mut Store) -> Self {
        Self {
            from: self.to,
            to: self.from,
            ..self
        }
    }

    fn split<I: IntNumber>(self, context: EdgeDataSplit<I>, _: &mut Store) -> (Self, Self) {
        let middle = self.from + (self.to - self.from) * split_share(&context);
        (
            Self { to: middle, ..self },
            Self {
                from: middle,
                ..self
            },
        )
    }

    fn merge(context: EdgeDataMerge<ShapeCountBoolean, Self>, store: &mut Store) -> Self {
        let mut pieces = Vec::new();
        for side in [context.lhs_data, context.rhs_data] {
            store
                .consumed
                .entry(side.group)
                .or_default()
                .push((side.from.min(side.to), side.from.max(side.to)));
            pieces.extend(store.pieces(side));
        }
        let data = store.group(pieces);
        store.merged.push(data.group);
        data
    }
}

/// An integer point of the overlay.
type Int = IntPoint<i32>;

/// An edge of a contour between passes: from `a` to `b`, the floor on its left, and the pieces
/// it carries.
#[derive(Debug, Clone, Copy)]
struct Edge {
    /// Its start.
    a: Int,
    /// Its end.
    b: Int,
    /// What it carries, with the floor on the left of every piece.
    data: EdgeData,
}

/// A run of coincident pieces that two outlines share, left inside the floor of the pass that
/// found it, and the number of the last outline that pass took in.
#[derive(Debug, Clone)]
struct Shared {
    /// The pieces lying along it, with the base segment from its start to its end.
    pieces: Vec<Piece>,
    /// The last outline the pass took in.
    after: usize,
}

/// The folding of a Layer's closed outlines through the overlay.
struct Fold<'a> {
    /// Every outline of the Layer.
    outlines: &'a [Outline],
    /// The closed outlines, by number, in stacking order.
    rooms: &'a [usize],
    /// The adapter between the Level's cells and the overlay's integers.
    adapter: FloatPointAdapter<[f64; 2], i32>,
    /// The groups of pieces.
    store: Store,
    /// Each closed outline's own contours, by number, the floor it adds on their left.
    own: BTreeMap<usize, Vec<Vec<Edge>>>,
    /// The combined floor's contours, by shape, after the last pass.
    floor: Vec<Vec<Vec<Edge>>>,
    /// The runs of shared pieces every pass found.
    shared: Vec<Shared>,
    /// The groups of outlines that touch along an edge.
    touching: Vec<Vec<usize>>,
}

impl<'a> Fold<'a> {
    /// The fold of `rooms`, the closed outlines among `outlines`, each brought to its own
    /// contours with the floor it adds on their left.
    fn new(outlines: &'a [Outline], rooms: &'a [usize]) -> Self {
        let flattened: BTreeMap<usize, Vec<Vec<(f64, Point)>>> = rooms
            .iter()
            .map(|index| (*index, parts_flattened(&outlines[*index].path)))
            .collect();
        let corners: Vec<[f64; 2]> = flattened
            .values()
            .flatten()
            .flatten()
            .map(|(_, at)| [at.x, at.y])
            .collect();
        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_iter(corners.iter());
        let mut fold = Self {
            outlines,
            rooms,
            adapter,
            store: Store::default(),
            own: BTreeMap::new(),
            floor: Vec::new(),
            shared: Vec::new(),
            touching: Vec::new(),
        };
        for (index, parts) in flattened {
            let cuts = outlines[index].cuts;
            let mut edges = Vec::new();
            for (part, points) in parts.iter().enumerate() {
                for pair in points.windows(2) {
                    let piece = Piece {
                        outline: index,
                        part,
                        t0: pair[0].0,
                        t1: pair[1].0,
                        left: true,
                        cuts,
                    };
                    let data = fold.store.group(vec![piece]);
                    edges.push((
                        fold.adapter.float_to_int(&[pair[0].1.x, pair[0].1.y]),
                        fold.adapter.float_to_int(&[pair[1].1.x, pair[1].1.y]),
                        data,
                        ShapeType::Subject,
                    ));
                }
            }
            let shapes = fold.pass(&edges, OverlayRule::Subject, None);
            fold.own
                .insert(index, shapes.into_iter().flatten().collect());
        }
        fold
    }

    /// One pass of the overlay over `edges` under `rule`, its result's contours by shape with the
    /// floor on the left of each edge. The shared pieces it finds are kept, with `after` the last
    /// outline it took in, when it is given.
    fn pass(
        &mut self,
        edges: &[(Int, Int, EdgeData, ShapeType)],
        rule: OverlayRule,
        after: Option<usize>,
    ) -> Vec<Vec<Vec<Edge>>> {
        let mut overlay: EdgeOverlay<i32, EdgeData> = EdgeOverlay::new(edges.len());
        let store = overlay.data_store_mut();
        std::mem::swap(store, &mut self.store);
        store.merged.clear();
        store.consumed.clear();
        for &(a, b, data, shape) in edges {
            overlay.add_edge(InputEdge { a, b, data }, shape);
        }
        let shapes = overlay.build_vector_shapes(rule, FillRule::NonZero);
        self.store = overlay.into_data_store();
        self.collect_merges(after);
        shapes
            .into_iter()
            .map(|shape| {
                shape
                    .into_iter()
                    .map(|contour| {
                        contour
                            .into_iter()
                            .map(|edge| {
                                let left = floor_on_left(edge.fill, rule);
                                let pieces = self
                                    .store
                                    .pieces(edge.data)
                                    .into_iter()
                                    .map(|piece| Piece { left, ..piece })
                                    .collect();
                                Edge {
                                    a: edge.a,
                                    b: edge.b,
                                    data: self.store.group(pieces),
                                }
                            })
                            .collect()
                    })
                    .collect()
            })
            .collect()
    }

    /// Keeps what the merges of the pass just run found: the outlines that touch, and, when the
    /// pass combines outlines (`after` is given), the runs of pieces no later merge took up that
    /// two outlines that do not cut share from opposite sides.
    fn collect_merges(&mut self, after: Option<usize>) {
        let merged = std::mem::take(&mut self.store.merged);
        let consumed = std::mem::take(&mut self.store.consumed);
        for group in merged {
            let Some(pieces) = self.store.groups.get(group) else {
                continue;
            };
            let mut outlines: Vec<usize> = pieces.iter().map(|piece| piece.outline).collect();
            outlines.sort_unstable();
            outlines.dedup();
            if outlines.len() > 1 {
                self.touching.push(outlines);
            }
            let Some(after) = after else {
                continue;
            };
            if !opposite(pieces) {
                continue;
            }
            let taken = consumed.get(&group).cloned().unwrap_or_default();
            for (from, to) in free(taken) {
                let pieces = self.store.pieces(EdgeData { group, from, to });
                self.shared.push(Shared { pieces, after });
            }
        }
    }

    /// Folds the closed outlines in stacking order: each run of outlines that do not cut is a
    /// union onto the floor so far, each run that cuts a difference from it.
    fn run(&mut self) {
        let rooms = self.rooms;
        let mut floor: Vec<Vec<Vec<Edge>>> = Vec::new();
        let mut start = 0;
        while start < rooms.len() {
            let cuts = self.outlines[rooms[start]].cuts;
            let mut end = start;
            while end + 1 < rooms.len() && self.outlines[rooms[end + 1]].cuts == cuts {
                end += 1;
            }
            let mut edges: Vec<(Int, Int, EdgeData, ShapeType)> = floor
                .iter()
                .flatten()
                .flatten()
                .map(|edge| (edge.a, edge.b, edge.data, ShapeType::Subject))
                .collect();
            let shape = if cuts {
                ShapeType::Clip
            } else {
                ShapeType::Subject
            };
            for index in &rooms[start..=end] {
                if let Some(contours) = self.own.get(index) {
                    edges.extend(
                        contours
                            .iter()
                            .flatten()
                            .map(|edge| (edge.a, edge.b, edge.data, shape)),
                    );
                }
            }
            floor = if cuts {
                self.pass(&edges, OverlayRule::Difference, None)
            } else {
                self.pass(&edges, OverlayRule::Subject, Some(rooms[end]))
            };
            start = end + 1;
        }
        self.floor = floor;
    }

    /// The point of the Level an integer point of the overlay stands for.
    fn point(&self, at: Int) -> Point {
        let [x, y] = self.adapter.int_to_float(&at);
        Point::new(x, y)
    }

    /// Each closed outline's floor: none for one that cuts, what its line winds around when no
    /// later outline that cuts reaches it, and otherwise its own contours less those of the later
    /// outlines that cut.
    fn floors(&mut self, lines: &[Vec<LinePoint>], combined: &mut [CombinedOutline]) {
        let boxes: BTreeMap<usize, Option<(Point, Point)>> = self
            .rooms
            .iter()
            .map(|index| (*index, bounds(&lines[*index])))
            .collect();
        for (place, index) in self.rooms.iter().enumerate() {
            let index = *index;
            if self.outlines[index].cuts {
                continue;
            }
            let reaching: Vec<usize> = self.rooms[place + 1..]
                .iter()
                .copied()
                .filter(|later| {
                    self.outlines[*later].cuts
                        && overlap(boxes[&index], boxes.get(later).copied().flatten())
                })
                .collect();
            if reaching.is_empty() {
                combined[index].floor = fill(&lines[index]);
                continue;
            }
            let mut edges: Vec<(Int, Int, EdgeData, ShapeType)> = Vec::new();
            for (outline, shape) in std::iter::once((index, ShapeType::Subject))
                .chain(reaching.iter().map(|later| (*later, ShapeType::Clip)))
            {
                if let Some(contours) = self.own.get(&outline) {
                    edges.extend(
                        contours
                            .iter()
                            .flatten()
                            .map(|edge| (edge.a, edge.b, edge.data, shape)),
                    );
                }
            }
            let left = self.pass(&edges, OverlayRule::Difference, None);
            let contours: Vec<Vec<Vec2>> = left
                .iter()
                .flatten()
                .map(|contour| {
                    contour
                        .iter()
                        .map(|edge| vector(self.point(edge.a)))
                        .collect()
                })
                .collect();
            combined[index].floor = fill_contours(&contours);
        }
    }

    /// Which closed outlines form one combination, its last outline written on each: those whose
    /// floors overlap or touch, cuts included, found from the shapes of their union and the
    /// edges they share.
    fn combinations(&mut self, combined: &mut [CombinedOutline]) {
        let any_cut = self.rooms.iter().any(|index| self.outlines[*index].cuts);
        let union = if any_cut {
            let edges: Vec<(Int, Int, EdgeData, ShapeType)> = self
                .own
                .values()
                .flatten()
                .flatten()
                .map(|edge| (edge.a, edge.b, edge.data, ShapeType::Subject))
                .collect();
            self.pass(&edges, OverlayRule::Subject, None)
        } else {
            self.floor.clone()
        };
        let mut sets = Sets::new(combined.len());
        let mut found = vec![false; combined.len()];
        for group in &self.touching {
            for pair in group.windows(2) {
                sets.join(pair[0], pair[1]);
            }
        }
        let mut firsts = Vec::with_capacity(union.len());
        for shape in &union {
            let mut first: Option<usize> = None;
            for edge in shape.iter().flatten() {
                for piece in self.store.pieces(edge.data) {
                    found[piece.outline] = true;
                    match first {
                        Some(first) => sets.join(first, piece.outline),
                        None => first = Some(piece.outline),
                    }
                }
            }
            firsts.push(first);
        }
        for index in self.rooms {
            if found[*index] {
                continue;
            }
            // An outline none of whose edges lies on the union's boundary lies inside one of its
            // shapes: the middle of its first edge tells which.
            let Some(edge) = self
                .own
                .get(index)
                .and_then(|contours| contours.first()?.first())
            else {
                continue;
            };
            let (a, b) = (self.point(edge.a), self.point(edge.b));
            let middle = a.midpoint(b);
            let within = union
                .iter()
                .zip(&firsts)
                .find(|(shape, _)| self.winds_around(shape, middle));
            if let Some((_, Some(first))) = within {
                sets.join(*first, *index);
            }
        }
        let mut last: BTreeMap<usize, usize> = BTreeMap::new();
        for index in self.rooms {
            let root = sets.root(*index);
            let entry = last.entry(root).or_insert(*index);
            *entry = (*entry).max(*index);
        }
        for index in self.rooms {
            combined[*index].last = last[&sets.root(*index)];
        }
    }

    /// Whether the contours of a shape wind around `at`, by the non-zero rule.
    fn winds_around(&self, shape: &[Vec<Edge>], at: Point) -> bool {
        let mut winding = 0_i32;
        for edge in shape.iter().flatten() {
            let (a, b) = (self.point(edge.a), self.point(edge.b));
            let left = (b - a).cross(at - a);
            if a.y <= at.y {
                if b.y > at.y && left > 0.0 {
                    winding += 1;
                }
            } else if b.y <= at.y && left < 0.0 {
                winding -= 1;
            }
        }
        winding != 0
    }

    /// The Walls: one closed chain round each contour of the combined floor, and chains along
    /// the edges Rooms share, clipped to the combined floor where a later cut reaches them.
    fn walls(&self) -> (Vec<Chain>, Vec<Chain>) {
        let round: Vec<Chain> = self
            .floor
            .iter()
            .flatten()
            .map(|contour| Chain {
                closed: true,
                chords: contour
                    .iter()
                    .map(|edge| {
                        chord(
                            self.point(edge.a),
                            self.point(edge.b),
                            self.store.pieces(edge.data),
                        )
                    })
                    .collect(),
            })
            .collect();
        let last_cut = self
            .rooms
            .iter()
            .rev()
            .find(|index| self.outlines[**index].cuts)
            .copied();
        let mut pieces: Vec<(Point, Point, Vec<Piece>)> = Vec::new();
        let mut to_clip: Vec<(Point, Point, Vec<Piece>)> = Vec::new();
        for shared in &self.shared {
            let Some(first) = shared.pieces.iter().find(|piece| !piece.cuts) else {
                continue;
            };
            let path = &self.outlines[first.outline].path;
            let (a, b) = (
                exact(path, first.part, first.t0),
                exact(path, first.part, first.t1),
            );
            if last_cut.is_some_and(|cut| cut > shared.after) {
                to_clip.push((a, b, shared.pieces.clone()));
            } else {
                pieces.push((a, b, shared.pieces.clone()));
            }
        }
        pieces.extend(self.clipped(&to_clip));
        let shared = chains_of(self.with_coincident(pieces));
        (round, shared)
    }

    /// The shared runs with the pieces of every other outline whose edge lies along them, each
    /// run cut where such an edge begins or ends. The overlay drops two coincident edges that
    /// cancel before it meets a third along them, so the third is found here.
    fn with_coincident(
        &self,
        runs: Vec<(Point, Point, Vec<Piece>)>,
    ) -> Vec<(Point, Point, Vec<Piece>)> {
        let mut found = Vec::with_capacity(runs.len());
        for (a, b, pieces) in runs {
            let (low, high) = (
                Point::new(a.x.min(b.x), a.y.min(b.y)),
                Point::new(a.x.max(b.x), a.y.max(b.y)),
            );
            let mut extra: Vec<(f64, f64, Vec<Piece>)> = Vec::new();
            for (outline, contours) in &self.own {
                if pieces.iter().any(|piece| piece.outline == *outline) {
                    continue;
                }
                for edge in contours.iter().flatten() {
                    let (p, q) = (self.point(edge.a), self.point(edge.b));
                    let reaches = p.x.max(q.x) + SAME_POINT >= low.x
                        && p.x.min(q.x) - SAME_POINT <= high.x
                        && p.y.max(q.y) + SAME_POINT >= low.y
                        && p.y.min(q.y) - SAME_POINT <= high.y;
                    if !reaches {
                        continue;
                    }
                    let Some((from, to)) = along(a, b, p, q) else {
                        continue;
                    };
                    let (from, to) = (from.min(to), from.max(to));
                    let squared = (q - p).hypot2();
                    if squared <= 0.0 {
                        continue;
                    }
                    let share = |s: f64| (q - p).dot(a.lerp(b, s) - p) / squared;
                    let base = EdgeLike {
                        pieces: &self.store.pieces(edge.data),
                    }
                    .between(share(from), share(to));
                    extra.push((from, to, base));
                }
            }
            if extra.is_empty() {
                found.push((a, b, pieces));
                continue;
            }
            let mut cuts: Vec<f64> = vec![0.0, 1.0];
            cuts.extend(extra.iter().flat_map(|(from, to, _)| [*from, *to]));
            cuts.sort_by(f64::total_cmp);
            cuts.dedup_by(|later, earlier| *later - *earlier <= NO_PIECE);
            for pair in cuts.windows(2) {
                let (start, end) = (pair[0], pair[1]);
                if end - start <= NO_PIECE {
                    continue;
                }
                let mut sub = EdgeLike { pieces: &pieces }.between(start, end);
                for (from, to, more) in &extra {
                    if *from <= start + NO_PIECE && end <= *to + NO_PIECE {
                        let span = to - from;
                        sub.extend(
                            EdgeLike { pieces: more }
                                .between((start - from) / span, (end - from) / span),
                        );
                    }
                }
                found.push((a.lerp(b, start), a.lerp(b, end), sub));
            }
        }
        found
    }

    /// The parts of each shared run that lie inside the combined floor, its boundary left out.
    fn clipped(&self, runs: &[(Point, Point, Vec<Piece>)]) -> Vec<(Point, Point, Vec<Piece>)> {
        if runs.is_empty() {
            return Vec::new();
        }
        let contours: Vec<Vec<[f64; 2]>> = self
            .floor
            .iter()
            .flatten()
            .map(|contour| {
                contour
                    .iter()
                    .map(|edge| {
                        let at = self.point(edge.a);
                        [at.x, at.y]
                    })
                    .collect()
            })
            .collect();
        if contours.is_empty() {
            return Vec::new();
        }
        let strings: Vec<Vec<[f64; 2]>> = runs
            .iter()
            .map(|(a, b, _)| vec![[a.x, a.y], [b.x, b.y]])
            .collect();
        let kept = strings.clip_by(
            &contours,
            FillRule::NonZero,
            ClipRule {
                invert: false,
                boundary_included: false,
            },
        );
        let mut parts = Vec::new();
        for path in kept {
            for pair in path.windows(2) {
                let (p, q) = (
                    Point::new(pair[0][0], pair[0][1]),
                    Point::new(pair[1][0], pair[1][1]),
                );
                for (a, b, pieces) in runs {
                    let Some((from, to)) = along(*a, *b, p, q) else {
                        continue;
                    };
                    if (to - from).abs() <= NO_PIECE {
                        continue;
                    }
                    let base = EdgeLike { pieces };
                    parts.push((a.lerp(*b, from), a.lerp(*b, to), base.between(from, to)));
                }
            }
        }
        parts
    }
}

/// Pieces along a segment, cut down to a share of it.
struct EdgeLike<'a> {
    /// The pieces, with the segment as their base.
    pieces: &'a [Piece],
}

impl EdgeLike<'_> {
    /// The pieces between two shares of the segment.
    fn between(&self, from: f64, to: f64) -> Vec<Piece> {
        self.pieces
            .iter()
            .map(|piece| piece.between(from, to))
            .collect()
    }
}

/// The shares of the segment from `a` to `b` at which `p` and `q` lie, when both lie on it.
fn along(a: Point, b: Point, p: Point, q: Point) -> Option<(f64, f64)> {
    let direction = b - a;
    let squared = direction.hypot2();
    if squared <= 0.0 {
        return None;
    }
    let length = squared.sqrt();
    let off = |at: Point| (direction.cross(at - a) / length).abs();
    if off(p) > SAME_POINT || off(q) > SAME_POINT {
        return None;
    }
    let share = |at: Point| direction.dot(at - a) / squared;
    let (from, to) = (share(p), share(q));
    let (low, high) = (from.min(to).max(0.0), from.max(to).min(1.0));
    if high - low <= NO_PIECE {
        return None;
    }
    Some(if from <= to { (low, high) } else { (high, low) })
}

/// The ranges of shares from zero to one that none of `taken` covers.
fn free(taken: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    let mut free = Vec::new();
    let mut start = 0.0;
    for (from, to) in merged(taken) {
        if from - start > NO_PIECE {
            free.push((start, from));
        }
        start = f64::max(start, to);
    }
    if 1.0 - start > NO_PIECE {
        free.push((start, 1.0));
    }
    free
}

/// Whether two of `pieces`, of different outlines that do not cut, bound floor on opposite
/// sides.
fn opposite(pieces: &[Piece]) -> bool {
    let adding: Vec<&Piece> = pieces.iter().filter(|piece| !piece.cuts).collect();
    adding.iter().enumerate().any(|(index, first)| {
        adding[index + 1..]
            .iter()
            .any(|second| second.outline != first.outline && second.left != first.left)
    })
}

/// Whether the result of a pass under `rule` lies on the left of an edge of its contour, by the
/// fill of the subject and the clip on either side.
fn floor_on_left(fill: u8, rule: OverlayRule) -> bool {
    let kept = |subject: bool, clip: bool| match rule {
        OverlayRule::Difference => subject && !clip,
        OverlayRule::Subject
        | OverlayRule::Clip
        | OverlayRule::Intersect
        | OverlayRule::Union
        | OverlayRule::InverseDifference
        | OverlayRule::Xor => subject,
    };
    let left = kept(fill & SUBJ_LEFT != 0, fill & CLIP_LEFT != 0);
    let right = kept(fill & SUBJ_RIGHT != 0, fill & CLIP_RIGHT != 0);
    left || !right
}

/// A chord from `a` to `b` along `pieces`, drawn for the last of them.
fn chord(a: Point, b: Point, pieces: Vec<Piece>) -> Chord {
    let sources: Vec<Source> = pieces.iter().map(Piece::source).collect();
    let owner = sources
        .iter()
        .copied()
        .max_by_key(|source| source.outline)
        .unwrap_or(Source {
            outline: 0,
            part: 0,
            from: 0.0,
            to: 0.0,
        });
    Chord {
        a,
        b,
        owner,
        sources,
    }
}

/// The shared runs joined end to end into chains wherever one ends where another starts.
fn chains_of(runs: Vec<(Point, Point, Vec<Piece>)>) -> Vec<Chain> {
    let mut left: Vec<Chord> = runs
        .into_iter()
        .map(|(a, b, pieces)| chord(a, b, pieces))
        .collect();
    let mut chains = Vec::new();
    while let Some(first) = left.pop() {
        let mut chords = std::collections::VecDeque::from([first]);
        while let Some(end) = chords.back().map(|chord| chord.b) {
            let Some(next) = left.iter().position(|chord| {
                chord.a.distance(end) <= SAME_POINT || chord.b.distance(end) <= SAME_POINT
            }) else {
                break;
            };
            let next = left.swap_remove(next);
            chords.push_back(if next.a.distance(end) <= SAME_POINT {
                next
            } else {
                reversed(next)
            });
        }
        while let Some(start) = chords.front().map(|chord| chord.a) {
            let Some(previous) = left.iter().position(|chord| {
                chord.b.distance(start) <= SAME_POINT || chord.a.distance(start) <= SAME_POINT
            }) else {
                break;
            };
            let previous = left.swap_remove(previous);
            chords.push_front(if previous.b.distance(start) <= SAME_POINT {
                previous
            } else {
                reversed(previous)
            });
        }
        chains.push(Chain {
            closed: false,
            chords: chords.into(),
        });
    }
    chains
}

/// A chord run the other way.
fn reversed(chord: Chord) -> Chord {
    let turn = |source: Source| Source {
        from: source.to,
        to: source.from,
        ..source
    };
    Chord {
        a: chord.b,
        b: chord.a,
        owner: turn(chord.owner),
        sources: chord.sources.into_iter().map(turn).collect(),
    }
}

/// The point of part `part` of `path` at parameter `t`, on the exact curve, its ends exactly the
/// outline's points.
pub(crate) fn exact(path: &Path, part: usize, t: f64) -> Point {
    let Some((start, end)) = path.ends(part) else {
        return Point::ZERO;
    };
    if t <= 0.0 {
        return crate::wall::point(start);
    }
    if t >= 1.0 {
        return crate::wall::point(end);
    }
    match path.curve(part) {
        Some(Curve::Bent(quad)) => kurbo::ParamCurve::eval(&quad, t),
        Some(Curve::Straight(line)) => kurbo::ParamCurve::eval(&line, t),
        None => Point::ZERO,
    }
}

/// The lines drawn for a chain: each run of chords drawn for one outline, with that outline, a
/// chain that is one outline's all the way round staying closed.
fn wall_lines(paths: &[&Path], chain: &Chain) -> Vec<(usize, WallLine)> {
    let count = chain.chords.len();
    if count == 0 {
        return Vec::new();
    }
    let owners: Vec<usize> = chain
        .chords
        .iter()
        .map(|chord| chord.owner.outline)
        .collect();
    if chain.closed && owners.iter().all(|owner| *owner == owners[0]) {
        let chords: Vec<&Chord> = chain.chords.iter().collect();
        return vec![(owners[0], line_of(paths[owners[0]], &chords, true))];
    }
    // A closed chain starts where its owner changes, so no run wraps past its first chord.
    let start = if chain.closed {
        (0..count)
            .find(|index| owners[*index] != owners[(index + count - 1) % count])
            .unwrap_or(0)
    } else {
        0
    };
    let ordered: Vec<&Chord> = (0..count)
        .map(|offset| &chain.chords[(start + offset) % count])
        .collect();
    let mut lines = Vec::new();
    let mut run: Vec<&Chord> = Vec::new();
    for chord in ordered {
        if run
            .last()
            .is_some_and(|last| last.owner.outline != chord.owner.outline)
        {
            let owner = run[0].owner.outline;
            lines.push((owner, line_of(paths[owner], &run, false)));
            run.clear();
        }
        run.push(chord);
    }
    if let Some(first) = run.first() {
        let owner = first.owner.outline;
        lines.push((owner, line_of(paths[owner], &run, false)));
    }
    lines
}

/// The line of a run of chords drawn for one outline: its ends where the chords end, and every
/// point between on the outline's exact curve, given twice where the run passes from one part to
/// the next.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the model keeps parameters in single precision"
)]
fn line_of(path: &Path, run: &[&Chord], closed: bool) -> WallLine {
    let mut line: Vec<LinePoint> = Vec::with_capacity(run.len() * 2);
    let last = run.len() - 1;
    for (index, chord) in run.iter().enumerate() {
        let owner = chord.owner;
        let start = if index == 0 && !closed {
            chord.a
        } else {
            exact(path, owner.part, owner.from)
        };
        let end = if index == last && !closed {
            chord.b
        } else {
            exact(path, owner.part, owner.to)
        };
        let start = LinePoint {
            position: vector(start),
            segment: owner.part,
            t: owner.from as f32,
        };
        if line.last() != Some(&start) {
            line.push(start);
        }
        line.push(LinePoint {
            position: vector(end),
            segment: owner.part,
            t: owner.to as f32,
        });
    }
    WallLine { line, closed }
}

/// The smallest box around a line's points, if it has any.
fn bounds(line: &[LinePoint]) -> Option<(Point, Point)> {
    let mut points = line.iter().map(|point| crate::wall::point(point.position));
    let first = points.next()?;
    Some(points.fold((first, first), |(low, high), at| {
        (
            Point::new(low.x.min(at.x), low.y.min(at.y)),
            Point::new(high.x.max(at.x), high.y.max(at.y)),
        )
    }))
}

/// Whether two boxes overlap or touch.
fn overlap(first: Option<(Point, Point)>, second: Option<(Point, Point)>) -> bool {
    let (Some((low, high)), Some((other_low, other_high))) = (first, second) else {
        return false;
    };
    low.x <= other_high.x && other_low.x <= high.x && low.y <= other_high.y && other_low.y <= high.y
}

/// Disjoint sets of outlines, by number.
struct Sets {
    /// The parent of each.
    parents: Vec<usize>,
}

impl Sets {
    /// Every outline in a set of its own.
    fn new(count: usize) -> Self {
        Self {
            parents: (0..count).collect(),
        }
    }

    /// The outline that names the set `index` is in.
    fn root(&mut self, index: usize) -> usize {
        let mut root = index;
        while self.parents[root] != root {
            root = self.parents[root];
        }
        let mut at = index;
        while self.parents[at] != root {
            let next = self.parents[at];
            self.parents[at] = root;
            at = next;
        }
        root
    }

    /// Puts the sets of two outlines together.
    fn join(&mut self, first: usize, second: usize) {
        let (first, second) = (self.root(first), self.root(second));
        if first != second {
            self.parents[first.max(second)] = first.min(second);
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        reason = "a test stops at the first thing that is not as expected"
    )]

    use super::*;

    /// A closed outline through `points` with the given control points, cutting or not.
    fn room(points: &[[f32; 2]], controls: &[Option<[f32; 2]>], cuts: bool) -> Outline {
        let mut controls: Vec<Option<Vec2>> = controls
            .iter()
            .map(|control| control.map(Vec2::from))
            .collect();
        controls.resize(points.len(), None);
        Outline {
            path: Path {
                points: points.iter().copied().map(Vec2::from).collect(),
                controls,
                closed: true,
            },
            cuts,
        }
    }

    /// The square from `low` to `high`, counter-clockwise, every edge straight.
    fn square(low: [f32; 2], high: [f32; 2], cuts: bool) -> Outline {
        room(
            &[low, [high[0], low[1]], high, [low[0], high[1]]],
            &[],
            cuts,
        )
    }

    /// The ranges of parameters along `part` of outline `outline` that its own Walls run along,
    /// each from the lower to the higher, in order.
    fn drawn(combination: &Combination, outline: usize, part: usize) -> Vec<(f32, f32)> {
        let mut ranges: Vec<(f32, f32)> = combination.outlines[outline]
            .walls
            .iter()
            .flat_map(|wall| wall.line.windows(2))
            .filter(|pair| pair[0].segment == part && pair[1].segment == part)
            .filter(|pair| (pair[0].t - pair[1].t).abs() > 0.0)
            .map(|pair| (pair[0].t.min(pair[1].t), pair[0].t.max(pair[1].t)))
            .collect();
        ranges.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut joined: Vec<(f32, f32)> = Vec::new();
        for (from, to) in ranges {
            match joined.last_mut() {
                Some(last) if from <= last.1 + 1e-6 => last.1 = last.1.max(to),
                _ => joined.push((from, to)),
            }
        }
        joined
    }

    /// Whether two ranges of parameters are the same within a millionth.
    fn same(first: &[(f32, f32)], second: &[(f32, f32)]) -> bool {
        first.len() == second.len()
            && first
                .iter()
                .zip(second)
                .all(|(a, b)| (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6)
    }

    /// Whether `p` lies in a triangle of the floor.
    fn floored(floor: &FillMesh, p: Vec2) -> bool {
        floor.indices.chunks(3).any(|corners| {
            let [a, b, c] = [0, 1, 2].map(|i| floor.vertices[corners[i] as usize]);
            let side = |u: Vec2, v: Vec2| (v - u).perp_dot(p - u);
            let (ab, bc, ca) = (side(a, b), side(b, c), side(c, a));
            (ab >= 0.0 && bc >= 0.0 && ca >= 0.0) || (ab <= 0.0 && bc <= 0.0 && ca <= 0.0)
        })
    }

    /// Through a union and a difference every Wall is a piece of one outline's edge, with that
    /// edge's number and the parameters along it: two overlapping squares keep the parts of
    /// their edges outside each other, and a cut across both is walled by its own edges where it
    /// takes floor away.
    #[test]
    fn pieces_keep_their_source_through_a_cut() {
        let combination = combine_outlines(&[
            square([0.0, 0.0], [4.0, 4.0], false),
            square([2.0, 2.0], [6.0, 6.0], false),
            square([3.0, -1.0], [5.0, 3.0], true),
        ]);

        // The first square's right edge (4,0)→(4,4) is inside the second above y = 2 and inside
        // the cut below y = 3: nothing of it is walled. Its bottom edge runs to x = 3, where the
        // cut begins.
        assert!(drawn(&combination, 0, 1).is_empty());
        assert!(same(&drawn(&combination, 0, 0), &[(0.0, 0.75)]));
        assert!(same(&drawn(&combination, 0, 2), &[(0.5, 1.0)]));
        assert!(same(&drawn(&combination, 0, 3), &[(0.0, 1.0)]));
        // The second square's bottom edge (2,2)→(6,2) is walled only from x = 5, past the cut;
        // its left edge (2,6)→(2,2) above y = 4.
        assert!(same(&drawn(&combination, 1, 0), &[(0.75, 1.0)]));
        assert!(same(&drawn(&combination, 1, 3), &[(0.0, 0.5)]));
        // The cut is walled where it takes floor away beside it: along its left edge
        // (3,3)→(3,-1) from y = 3 down to y = 0, the first square's floor beside it; along its
        // whole top edge (5,3)→(3,3), under one square's floor or the other's; and along its
        // right edge (5,-1)→(5,3) from y = 2 up, the second square's floor beside it.
        assert!(same(&drawn(&combination, 2, 3), &[(0.0, 0.75)]));
        assert!(same(&drawn(&combination, 2, 2), &[(0.0, 1.0)]));
        assert!(same(&drawn(&combination, 2, 1), &[(0.75, 1.0)]));
        assert!(
            drawn(&combination, 2, 0).is_empty(),
            "nothing below the floor"
        );
        for outline in &combination.outlines {
            assert_eq!(outline.last, 2, "one combination, the cut last");
        }
    }

    /// A curved edge combined with another outline keeps its Wall on its exact curve: every
    /// point of its Wall's line lies on the curve at the parameter it is tagged with, and the
    /// line ends where the other outline's edge crosses it.
    #[test]
    fn curved_pieces_are_rebuilt_on_their_curve() {
        let control = [5.0, 14.0];
        let combination = combine_outlines(&[
            room(
                &[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]],
                &[None, None, Some(control)],
                false,
            ),
            square([6.0, 8.0], [14.0, 16.0], false),
        ]);
        let curve = kurbo::QuadBez::new(
            Point::new(10.0, 10.0),
            Point::new(5.0, 14.0),
            Point::new(0.0, 10.0),
        );
        let on_curve: Vec<&LinePoint> = combination.outlines[0]
            .walls
            .iter()
            .flat_map(|wall| &wall.line)
            .filter(|point| point.segment == 2)
            .collect();
        assert!(on_curve.len() > 10, "the curve is flattened finely");
        let mut ends = 0;
        for point in &on_curve {
            let exact = vector(kurbo::ParamCurve::eval(&curve, f64::from(point.t)));
            let off = point.position.distance(exact);
            if off > 1e-5 {
                // Only where the square's edge x = 6 crosses the curve, within the tolerance.
                ends += 1;
                assert!((point.position.x - 6.0).abs() < 1e-5, "{point:?}");
                assert!(
                    f64::from(off) <= crate::path::TOLERANCE,
                    "{point:?} off by {off}"
                );
            }
        }
        assert!(ends <= 1, "{ends} points off the curve");
        let walled = drawn(&combination, 0, 2);
        assert_eq!(walled.len(), 1);
        let cut_at = walled[0].0;
        let crossing = kurbo::ParamCurve::eval(&curve, f64::from(cut_at));
        assert!(
            (crossing.x - 6.0).abs() < 2e-3,
            "ends at x = 6: {crossing:?}"
        );
        assert!(
            (walled[0].1 - 1.0).abs() < 1e-6,
            "and runs to the curve's end"
        );
    }

    /// An outline after a cut adds floor as any other, its Walls carrying its own edges through
    /// the pass after the cut, and the floor of the outline before the cut leaves out what the
    /// cut takes away.
    #[test]
    fn a_room_after_a_cut_keeps_its_source() {
        let combination = combine_outlines(&[
            square([0.0, 0.0], [10.0, 10.0], false),
            square([2.0, 2.0], [8.0, 8.0], true),
            square([4.0, 4.0], [6.0, 12.0], false),
        ]);
        // The later room fills part of the hole and runs out through the top: its left edge
        // (4,12)→(4,4) is walled inside the hole, y 8 down to 4, and outside, y 12 down to 10.
        assert!(same(&drawn(&combination, 2, 3), &[(0.0, 0.25), (0.5, 1.0)]));
        // Its bottom edge lies in the hole: walled whole.
        assert!(same(&drawn(&combination, 2, 0), &[(0.0, 1.0)]));
        // The cut's bottom edge (2,2)→(8,2) is walled whole; its top edge (8,8)→(2,8) except
        // where the later room covers it, x 6 to 4.
        assert!(same(&drawn(&combination, 1, 0), &[(0.0, 1.0)]));
        assert!(same(
            &drawn(&combination, 1, 2),
            &[(0.0, 1.0 / 3.0), (2.0 / 3.0, 1.0)]
        ));
        let first = &combination.outlines[0].floor;
        assert!(floored(first, Vec2::new(1.0, 1.0)));
        assert!(!floored(first, Vec2::new(5.0, 5.0)), "the hole");
        assert!(!floored(first, Vec2::new(3.0, 5.0)), "the hole");
        let later = &combination.outlines[2].floor;
        assert!(
            floored(later, Vec2::new(5.0, 5.0)),
            "the later room fills it"
        );
        assert!(
            combination.outlines[1].floor.indices.is_empty(),
            "a cut has no floor"
        );
    }

    /// The Wall between two Rooms that share an edge is kept from the merge of their edges, and a
    /// later cut across it leaves only the parts with floor on both sides.
    #[test]
    fn a_shared_edge_is_clipped_by_a_cut() {
        let combination = combine_outlines(&[
            square([0.0, 0.0], [4.0, 4.0], false),
            square([4.0, 0.0], [8.0, 4.0], false),
            square([3.0, 1.0], [5.0, 2.0], true),
        ]);
        // The shared edge x = 4: the first square's right edge (4,0)→(4,4) and the second's left
        // edge (4,4)→(4,0). The second is the later, so the Wall is drawn for it, from y = 0 to
        // y = 1 and from y = 2 to y = 4.
        assert!(
            drawn(&combination, 0, 1).is_empty(),
            "drawn once, for the later"
        );
        assert!(same(&drawn(&combination, 1, 3), &[(0.0, 0.5), (0.75, 1.0)]));
        let walled: Vec<(f32, f32)> = combination.outlines[0]
            .walled
            .iter()
            .filter(|stretch| stretch.start.segment == 1)
            .map(|stretch| (stretch.start.t, stretch.end.t))
            .collect();
        assert!(same(&walled, &[(0.0, 0.25), (0.5, 1.0)]), "{walled:?}");
    }
}
