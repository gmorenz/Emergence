//! Signals are used for pathfinding and decision-making.
//!
//! By collecting information about the local environment into a slowly updated, tile-centric data structure,
//! we can scale path-finding and decisionmaking in a clear and comprehensible way.

use bevy::{prelude::*, utils::HashMap};
use core::fmt::Display;
use core::ops::{Add, Mul, Sub};
use itertools::Itertools;

use crate::units::behavior::Goal;
use crate::{
    items::ItemId,
    simulation::geometry::{MapGeometry, TilePos},
    structures::StructureId,
};

/// The resources and systems need to work with signals
pub(crate) struct SignalsPlugin;

impl Plugin for SignalsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Signals>()
            .add_system_to_stage(CoreStage::PreUpdate, emit_signals.before(diffuse_signals))
            .add_system_to_stage(
                CoreStage::PreUpdate,
                diffuse_signals.before(degrade_signals),
            )
            .add_system_to_stage(CoreStage::PreUpdate, degrade_signals);
    }
}

/// The central resource that tracks all signals.
#[derive(Resource, Debug, Default)]
pub(crate) struct Signals {
    /// The spatialized map for each signal
    maps: HashMap<SignalType, SignalMap>,
}

impl Signals {
    /// Returns the signal strength of `signal_type` at the given `tile_pos`.
    ///
    /// Missing values will be filled with [`SignalStrength::ZERO`].
    fn get(&self, signal_type: SignalType, tile_pos: TilePos) -> SignalStrength {
        match self.maps.get(&signal_type) {
            Some(map) => map.get(tile_pos),
            None => SignalStrength::ZERO,
        }
    }

    /// Adds `signal_strength` of `signal_type` at `tile_pos`.
    fn add_signal(
        &mut self,
        signal_type: SignalType,
        tile_pos: TilePos,
        signal_strength: SignalStrength,
    ) {
        match self.maps.get_mut(&signal_type) {
            Some(map) => map.add_signal(tile_pos, signal_strength),
            None => {
                let mut new_map = SignalMap::default();
                new_map.add_signal(tile_pos, signal_strength);
                self.maps.insert(signal_type, new_map);
            }
        }
    }

    /// Returns the complete set of signals at the given `tile_pos`.
    ///
    /// This is useful for decision-making.
    pub(crate) fn all_signals_at_position(&self, tile_pos: TilePos) -> LocalSignals {
        let mut all_signals = HashMap::new();
        for &signal_type in self.maps.keys() {
            let strength = self.get(signal_type, tile_pos);
            all_signals.insert(signal_type, strength);
        }

        LocalSignals { map: all_signals }
    }

    /// Returns the adjacent, empty tile position that contains the highest sum signal strength that can be used to meet the provided `goal`.
    ///
    /// If no suitable tile exists, [`None`] will be returned instead.
    pub(crate) fn upstream(
        &self,
        tile_pos: TilePos,
        goal: &Goal,
        map_geometry: &MapGeometry,
    ) -> Option<TilePos> {
        let possible_tiles = tile_pos.empty_neighbors(map_geometry);

        let mut best_choice = None;
        let mut best_score = SignalStrength::ZERO;

        let neighboring_signals = match goal {
            Goal::Wander => return None,
            Goal::Pickup(item_id) => {
                let push_signals =
                    self.neighboring_signals(SignalType::Push(*item_id), tile_pos, map_geometry);
                let contains_signals = self.neighboring_signals(
                    SignalType::Contains(*item_id),
                    tile_pos,
                    map_geometry,
                );
                let mut total_signals = push_signals;

                for (tile_pos, signal_strength) in contains_signals {
                    if let Some(existing_signal_strength) = total_signals.get_mut(&tile_pos) {
                        *existing_signal_strength = *existing_signal_strength + signal_strength;
                    } else {
                        total_signals.insert(tile_pos, signal_strength);
                    }
                }

                total_signals
            }
            Goal::DropOff(item_id) => {
                self.neighboring_signals(SignalType::Pull(*item_id), tile_pos, map_geometry)
            }
            Goal::Work(structure_id) => {
                self.neighboring_signals(SignalType::Work(*structure_id), tile_pos, map_geometry)
            }
        };

        for possible_tile in possible_tiles {
            if let Some(&current_score) = neighboring_signals.get(&possible_tile) {
                if current_score > best_score {
                    best_score = current_score;
                    best_choice = Some(possible_tile);
                }
            }
        }

        best_choice
    }

    /// Returns the signal strength of the type `signal_type` in `tile_pos` and its 6 surrounding neighbors.
    fn neighboring_signals(
        &self,
        signal_type: SignalType,
        tile_pos: TilePos,
        map_geometry: &MapGeometry,
    ) -> HashMap<TilePos, SignalStrength> {
        let mut signal_strength_map = HashMap::with_capacity(7);

        signal_strength_map.insert(tile_pos, self.get(signal_type, tile_pos));
        for neighbor in tile_pos.neighbors(map_geometry) {
            signal_strength_map.insert(neighbor, self.get(signal_type, neighbor));
        }

        signal_strength_map
    }
}

/// All of the signals on a single tile.
#[derive(Debug)]
pub(crate) struct LocalSignals {
    /// Internal data storage
    map: HashMap<SignalType, SignalStrength>,
}

impl LocalSignals {
    /// Returns the set of signals that might be used to pick a goal
    pub(crate) fn goal_relevant_signals(
        &self,
    ) -> impl Iterator<Item = (&SignalType, &SignalStrength)> + Clone {
        self.map.iter().filter(|(signal_type, _signal_strength)| {
            !matches!(**signal_type, SignalType::Contains(_))
        })
    }
}

impl Display for LocalSignals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::default();

        for signal_type in self.map.keys().sorted() {
            let signal_strength = self.map.get(signal_type).unwrap().0;

            let substring = format!("{signal_type}: {signal_strength:.2}\n");

            string += &substring;
        }

        write!(f, "{string}")
    }
}

/// Stores the [`SignalStrength`] of the given [`SignalType`] at each [`TilePos`].
#[derive(Debug, Default)]
struct SignalMap {
    /// The lookup data structure
    map: HashMap<TilePos, SignalStrength>,
}

impl SignalMap {
    /// Returns the signal strenth at the given [`TilePos`].
    ///
    /// Missing values will be filled with [`SignalStrength::ZERO`].
    fn get(&self, tile_pos: TilePos) -> SignalStrength {
        *self.map.get(&tile_pos).unwrap_or(&SignalStrength::ZERO)
    }

    /// Adds the `signal_strength` to the signal at `tile_pos`.
    fn add_signal(&mut self, tile_pos: TilePos, signal_strength: SignalStrength) {
        let existing = self.get(tile_pos);
        self.map.insert(tile_pos, existing + signal_strength);
    }

    /// Subtracts the `signal_strength` to the signal at `tile_pos`.
    ///
    /// The value is capped a minimum of [`SignalStrength::ZERO`].
    fn subtract_signal(&mut self, tile_pos: TilePos, signal_strength: SignalStrength) {
        let existing = self.get(tile_pos);
        self.map.insert(tile_pos, existing - signal_strength);
    }
}

/// The variety of signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum SignalType {
    /// Take this item away from here.
    Push(ItemId),
    /// Bring me an item of this type.
    Pull(ItemId),
    /// Has an item of this type, in case you were looking.
    Contains(ItemId),
    /// Perform work at this type of structure.
    #[allow(dead_code)]
    Work(StructureId),
}

impl Display for SignalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            SignalType::Push(item_id) => format!("Push({item_id})"),
            SignalType::Pull(item_id) => format!("Pull({item_id})"),
            SignalType::Contains(item_id) => format!("Contains({item_id})"),
            SignalType::Work(structure_id) => format!("Work({structure_id})"),
        };

        write!(f, "{string}")
    }
}

/// How strong a signal is.
///
/// This has a minimum value of 0.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct SignalStrength(f32);

impl SignalStrength {
    /// No signal is present.
    pub(crate) const ZERO: SignalStrength = SignalStrength(0.);

    /// Creates a new [`SignalStrength`], ensuring that it has a minimum value of 0.
    pub(crate) fn new(value: f32) -> Self {
        SignalStrength(value.max(0.))
    }

    /// The underlying value
    pub(crate) fn value(&self) -> f32 {
        self.0
    }
}

impl Add<SignalStrength> for SignalStrength {
    type Output = SignalStrength;

    fn add(self, rhs: SignalStrength) -> Self::Output {
        SignalStrength(self.0 + rhs.0)
    }
}

impl Sub<SignalStrength> for SignalStrength {
    type Output = SignalStrength;

    fn sub(self, rhs: SignalStrength) -> Self::Output {
        SignalStrength((self.0 - rhs.0).max(0.))
    }
}

impl Mul<f32> for SignalStrength {
    type Output = SignalStrength;

    fn mul(self, rhs: f32) -> Self::Output {
        SignalStrength(self.0 * rhs)
    }
}

/// The component that causes a game object to emit a signal.
///
/// This can change over time, and multiple signals may be emitted at once.
#[derive(Component, Debug, Default, Clone)]
pub(crate) struct Emitter {
    /// The list of signals to emit at a provided
    pub(crate) signals: Vec<(SignalType, SignalStrength)>,
}

/// Emits signals from [`Emitter`] sources.
fn emit_signals(mut signals: ResMut<Signals>, emitter_query: Query<(&TilePos, &Emitter)>) {
    for (&tile_pos, emitter) in emitter_query.iter() {
        for (signal_type, signal_strength) in &emitter.signals {
            signals.add_signal(*signal_type, tile_pos, *signal_strength);
        }
    }
}

/// Spreads signals between tiles.
fn diffuse_signals(
    mut signals: ResMut<Signals>,
    map_geometry: Res<MapGeometry>,
    mut pending_additions: Local<HashMap<SignalType, SignalMap>>,
    mut pending_removals: Local<HashMap<SignalType, SignalMap>>,
) {
    let map_geometry = &*map_geometry;

    /// The fraction of signals in each cell that will move to each of 6 neighbors each frame.
    ///
    /// Higher values will result in more spread out signals.
    ///
    /// If no neighbor exists, total diffusion will be reduced correspondingly.
    /// As a result, this value *must* be below 1/6,
    /// and probably should be below 1/7 to avoid weirdness.
    const DIFFUSION_FRACTION: f32 = 0.1;
    // These are scratch space:
    // reset them each time diffusion is run
    pending_additions.clear();
    pending_removals.clear();

    for (&signal_type, original_map) in signals.maps.iter() {
        let mut addition_map = SignalMap::default();
        let mut removal_map = SignalMap::default();

        for (&occupied_tile, original_strength) in original_map.map.iter() {
            let amount_to_send_to_each_neighbor = *original_strength * DIFFUSION_FRACTION;

            for neighboring_tile in occupied_tile.empty_neighbors(map_geometry) {
                removal_map.add_signal(occupied_tile, amount_to_send_to_each_neighbor);
                addition_map.add_signal(neighboring_tile, amount_to_send_to_each_neighbor);
            }
        }

        pending_additions.insert(signal_type, addition_map);
        pending_removals.insert(signal_type, removal_map);
    }

    // We cannot do this in one step, as we need to avoid bizarre iteration order dependencies
    for (signal_type, original_map) in signals.maps.iter_mut() {
        let addition_map = pending_additions.get(signal_type).unwrap();
        let removal_map = pending_additions.get(signal_type).unwrap();

        for (&removal_pos, &removal_strength) in removal_map.map.iter() {
            original_map.subtract_signal(removal_pos, removal_strength)
        }

        for (&addition_pos, &addition_strength) in addition_map.map.iter() {
            original_map.add_signal(addition_pos, addition_strength)
        }
    }
}

/// Degrades signals, allowing them to approach an asymptotically constant level.
fn degrade_signals(mut signals: ResMut<Signals>) {
    /// The fraction of signal that will decay at each step.
    ///
    /// Higher values lead to faster decay.
    /// This must always be between 0 and 1.
    const DEGRADATION_FRACTION: f32 = 0.1;

    for signal_map in signals.maps.values_mut() {
        for signal_strength in signal_map.map.values_mut() {
            *signal_strength = *signal_strength * (1. - DEGRADATION_FRACTION);
        }
    }
}