#![allow(dead_code)]

use std::f64::INFINITY;
use std::fmt::{self, Debug, Display};
use std::{collections::HashMap, error::Error};

use super::body::Body;
use glam::DVec3;
use keplerian_sim::{MuSetterMode, OrbitTrait};
use strum_macros::EnumIter;
pub type Id = u64;

const GRAVITATIONAL_CONSTANT: f64 = 6.6743e-11;

/// Struct that represents the simulation of the universe.
#[derive(Clone, Debug)]
pub struct Universe {
    /// The celestial bodies in the universe and their relations.
    bodies: HashMap<Id, BodyWrapper>,

    /// The next ID to assign to a body.
    next_id: Id,

    /// The time elapsed in the universe, in seconds.
    pub time: f64,

    /// The gravitational constant, in m^3 kg^-1 s^-2.
    g: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyRelation {
    pub parent: Option<Id>,
    pub satellites: Vec<Id>,
}

#[derive(Clone, Debug)]
pub struct BodyWrapper {
    pub body: Body,
    pub relations: BodyRelation,
}

#[derive(Clone, Debug)]
pub struct BodyAddError {
    cause: BodyAddErrorCause,
    body: Box<Body>,
}

#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum BodyAddErrorCause {
    ParentNotFound { parent_id: Id },
}

impl fmt::Display for BodyAddErrorCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BodyAddErrorCause::ParentNotFound { parent_id } => write!(
                f,
                "There was no body at the specified parent index of {parent_id}"
            ),
        }
    }
}

impl Error for BodyAddErrorCause {}

impl fmt::Display for BodyAddError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed to add body {:?} to the universe: {}",
            *self.body, self.cause
        )
    }
}

impl Error for BodyAddError {}

impl Universe {
    /// Creates an empty universe.
    pub fn new(g: Option<f64>) -> Universe {
        let g = g.unwrap_or(GRAVITATIONAL_CONSTANT);

        Universe {
            bodies: HashMap::new(),
            next_id: 0,
            time: 0.0,
            g,
        }
    }

    fn get_and_inc_id(&mut self) -> Id {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    /// Adds a body to the universe.
    ///
    /// `body`: The body to add into the universe.  
    /// `parent_id`: The index of the body that this body is orbiting.  
    /// Returns: The index of the newly-added body.  
    pub fn add_body(&mut self, mut body: Body, parent_id: Option<Id>) -> Result<Id, BodyAddError> {
        if let Some(parent_id) = parent_id {
            let parent = match self.bodies.get(&parent_id) {
                Some(b) => b,
                None => {
                    return Err(BodyAddError {
                        cause: BodyAddErrorCause::ParentNotFound { parent_id },
                        body: Box::new(body),
                    });
                }
            };

            if let Some(ref mut o) = body.orbit {
                o.set_gravitational_parameter(
                    self.g * parent.body.mass,
                    MuSetterMode::KeepElements,
                );
            }
        }

        let id = self.get_and_inc_id();

        self.bodies.insert(
            id,
            BodyWrapper {
                body,
                relations: BodyRelation {
                    parent: parent_id,
                    satellites: Vec::new(),
                },
            },
        );
        if let Some(parent_index) = parent_id {
            if let Some(wrapper) = self.bodies.get_mut(&parent_index) {
                wrapper.relations.satellites.push(id);
            }
        }

        Ok(id)
    }

    /// Removes a body from the universe.
    ///
    /// `body_index`: The index of the body to remove.
    ///
    /// Returns: A Vec of all bodies that were removed, including the one specified.  
    /// An empty Vec is returned if the body was not found.
    pub fn remove_body(&mut self, body_index: Id) -> Vec<(Id, Body)> {
        let wrapper = match self.bodies.remove(&body_index) {
            Some(wrapper) => wrapper,
            None => return Vec::new(),
        };

        let (body, relations) = (wrapper.body, wrapper.relations);
        let mut bodies = vec![(body_index, body)];

        // Remove the body from its parent's satellites.
        if let Some(parent_index) = relations.parent {
            if let Some(parent_wrapper) = self.bodies.get_mut(&parent_index) {
                parent_wrapper
                    .relations
                    .satellites
                    .retain(|&satellite| satellite != body_index);
            }
        }

        // Remove children
        for &satellite_index in &relations.satellites {
            bodies.append(&mut self.remove_body(satellite_index));
        }

        bodies
    }

    /// Gets a reference to a HashMap of all bodies in the universe and their relations.
    pub fn get_bodies(&self) -> &HashMap<Id, BodyWrapper> {
        &self.bodies
    }

    /// Gets a mutable reference to a body in the universe.
    pub fn get_body_mut(&mut self, index: Id) -> Option<&mut BodyWrapper> {
        self.bodies.get_mut(&index)
    }

    /// Gets an immutable reference to a body in the universe.
    pub fn get_body(&self, index: Id) -> Option<&BodyWrapper> {
        self.bodies.get(&index)
    }

    /// Gets the first index of a body with a given name, if any.
    pub fn get_body_index_with_name(&self, name: &str) -> Option<Id> {
        self.bodies
            .iter()
            .find(|(_, w)| w.body.name == name)
            .map(|(id, _)| *id)
    }

    pub fn tick(&mut self, dt: f64) {
        self.time += dt;
    }

    /// Gets the absolute position of a body in the universe.
    ///
    /// Each coordinate is in meters.
    ///
    /// `index`: The index of the body to get the position of.
    ///
    /// Returns: The absolute position of the body.  
    /// The top ancestor of the body (i.e, the body with no parent) is at the origin (0, 0, 0).  
    pub fn get_body_position(&self, index: Id) -> Option<DVec3> {
        let wrapper = self.bodies.get(&index)?;
        let (orbit, parent) = (&wrapper.body.orbit, wrapper.relations.parent);

        let mut position = match orbit {
            Some(orbit) => orbit.get_position_at_time(self.time),
            None => DVec3::ZERO, // If the body is not in orbit, its position is the origin
        };

        if let Some(parent) = parent {
            if let Some(parent_position) = self.get_body_position(parent) {
                position += parent_position;
            }
        }

        Some(position)
    }

    /// Gets the radius of the Sphere of Influence (SOI) of the body
    /// at the specified index.
    ///
    /// Returns None in some cases:
    /// - The body at the specified index was not found.
    /// - The parent was specified but a parent with that id was not found.
    pub fn get_soi_radius(&self, body_index: Id) -> Option<f64> {
        let wrapper = self.bodies.get(&body_index)?;

        let parent_id = match wrapper.relations.parent {
            Some(id) => id,
            None => return Some(INFINITY),
        };

        let orbit = match &wrapper.body.orbit {
            Some(id) => id,
            None => return Some(INFINITY),
        };

        let parent = self.bodies.get(&parent_id)?;
        let parent_mass = parent.body.mass;

        let body_mass = wrapper.body.mass;

        // Equation from https://en.wikipedia.org/wiki/Sphere_of_influence_(astrodynamics)
        // r_SOI \approx a (m/M)^(2/5)
        Some(orbit.get_semi_major_axis() * (body_mass / parent_mass).powf(2.0 / 5.0))
    }

    fn get_body_position_memoized(&self, index: Id, map: &mut HashMap<Id, DVec3>) -> Option<DVec3> {
        if let Some(&v) = map.get(&index) {
            return Some(v);
        }

        let wrapper = self.bodies.get(&index)?;
        let (orbit, parent) = (&wrapper.body.orbit, wrapper.relations.parent);

        let mut position = match orbit {
            Some(orbit) => orbit.get_position_at_time(self.time),
            None => DVec3::ZERO, // If the body is not in orbit, its position is the origin
        };

        if let Some(parent) = parent {
            if let Some(parent_position) = self.get_body_position_memoized(parent, map) {
                position += parent_position;
            }
        }

        map.insert(index, position);

        Some(position)
    }

    pub fn get_all_body_positions(&self) -> HashMap<Id, DVec3> {
        let mut map = HashMap::with_capacity(self.bodies.len());

        for &index in self.bodies.keys() {
            self.get_body_position_memoized(index, &mut map);
        }

        map
    }

    /// Duplicates a body and its satellites.
    ///
    /// Returns an Err if the body with the specified index was not found,
    /// or if the specified body is a root node.
    pub fn duplicate_body(&mut self, index: Id) -> Result<Id, ()> {
        let parent_index = match self.bodies.get(&index) {
            Some(w) => w.relations.parent,
            None => return Err(()),
        }
        .ok_or(())?;
        Ok(self.duplicate_body_inner(index, Some(parent_index)))
    }

    fn duplicate_body_inner(&mut self, index: Id, parent_index: Option<Id>) -> Id {
        let wrapper = self.get_body(index).unwrap();
        let body = wrapper.body.clone();
        let sats = wrapper.relations.satellites.clone();
        let new_index = self.add_body(body, parent_index).unwrap();

        for sat_index in sats {
            self.duplicate_body_inner(sat_index, Some(new_index));
        }
        new_index
    }

    #[inline]
    pub fn get_gravitational_constant(&self) -> f64 {
        self.g
    }

    pub fn set_gravitational_constant(&mut self, new_g: f64, mode: BulkMuSetterMode) {
        self.g = new_g;
        self.update_all_gravitational_parameters(mode);
    }

    /// Resynchronizes bodies' gravitational parameters to a calculated value.
    pub fn update_all_gravitational_parameters(&mut self, mode: BulkMuSetterMode) {
        let mode = mode.to_mu_setter(self.time);

        struct MuChange {
            body_id: Id,
            new_mu: f64,
        }

        let g = self.g;
        self.bodies
            .iter()
            .filter_map(|(&id, wrapper)| {
                let parent_id = wrapper.relations.parent?;
                let parent_mass = self.bodies.get(&parent_id)?.body.mass;
                let old_mu = wrapper.body.orbit.as_ref()?.get_gravitational_parameter();
                let new_mu = g * parent_mass;
                (old_mu != new_mu).then(|| MuChange {
                    body_id: id,
                    new_mu,
                })
            })
            .collect::<Box<[MuChange]>>()
            .into_iter()
            .for_each(|change| {
                let wrapper = match self.bodies.get_mut(&change.body_id) {
                    Some(w) => w,
                    None => return,
                };
                let orbit = match wrapper.body.orbit.as_mut() {
                    Some(o) => o,
                    None => return,
                };
                orbit.set_gravitational_parameter(change.new_mu, mode);
            });
    }

    pub fn update_children_gravitational_parameters(
        &mut self,
        parent_id: Id,
        mode: BulkMuSetterMode,
    ) -> Result<(), ()> {
        let mode = mode.to_mu_setter(self.time);

        let parent = self.bodies.get(&parent_id).ok_or(())?;

        let mu = parent.body.mass * self.g;

        parent
            .relations
            .satellites
            .iter()
            .copied()
            .collect::<Box<[Id]>>()
            .iter()
            .for_each(|child_id| {
                let wrapper = match self.bodies.get_mut(&child_id) {
                    Some(w) => w,
                    None => return,
                };
                let orbit = match &mut wrapper.body.orbit {
                    Some(o) => o,
                    None => return,
                };
                orbit.set_gravitational_parameter(mu, mode);
            });

        Ok(())
    }
}

impl Default for Universe {
    /// Creates an empty universe with default parameters.
    fn default() -> Universe {
        Universe {
            bodies: HashMap::new(),
            time: 0.0,
            g: GRAVITATIONAL_CONSTANT,
            next_id: 0,
        }
    }
}

/// A mode to describe how the gravitational parameter setter should behave.
///
/// This is used to describe how the setter should behave when setting the
/// gravitational parameter of the parent body.
///
/// # Which mode should I use?
/// The mode you should use depends on what you expect from setting the mu value
/// to a different value.
///
/// If you just want to set the mu value naïvely (without touching the
/// other orbital elements), you can use the `KeepElements` variant.
///
/// If you want to keep the current position
/// (not caring about the velocity), you can use the `KeepPosition` variant.
///
/// If you want to keep the current position and velocity, you can use the
/// `KeepStateVectors` mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EnumIter)]
pub enum BulkMuSetterMode {
    KeepElements,
    KeepPosition,
    #[default]
    KeepStateVectors,
}

impl BulkMuSetterMode {
    pub fn to_mu_setter(self, time: f64) -> MuSetterMode {
        match self {
            BulkMuSetterMode::KeepElements => MuSetterMode::KeepElements,
            BulkMuSetterMode::KeepPosition => MuSetterMode::KeepPositionAtTime(time),
            BulkMuSetterMode::KeepStateVectors => MuSetterMode::KeepStateVectorsAtTime(time),
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            BulkMuSetterMode::KeepElements => "Keep elements",
            BulkMuSetterMode::KeepPosition => "Keep positions",
            BulkMuSetterMode::KeepStateVectors => "Keep pos+vel",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            BulkMuSetterMode::KeepElements => {
                "Keep the current orbits' Keplerian elements.\n\
                This will change the position and velocity of the orbiting body abruptly.\n\
                It will not, however, change the trajectory of the bodies."
            }
            BulkMuSetterMode::KeepPosition => {
                "Keep the overall shape of the orbit(s), but make the bodies stay at the same position.\n\
                This will change the velocity of the orbiting body abruptly."
            }
            BulkMuSetterMode::KeepStateVectors => {
                "Keep the position and velocity of the orbit(s).\n\
                This will change the orbit's overall trajectory."
            }
        }
    }
}

impl Display for BulkMuSetterMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
