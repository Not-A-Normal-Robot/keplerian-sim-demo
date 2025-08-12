#![allow(dead_code)]

use std::fmt::{self, Debug};
use std::{collections::HashMap, error::Error};

use super::body::Body;
use glam::DVec3;
use keplerian_sim::{MuSetterMode, OrbitTrait};
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
    pub g: f64,
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
                    MuSetterMode::KeepPositionAtTime(self.time),
                );
            }
        }

        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);

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
    pub fn remove_body(&mut self, body_index: Id) -> Vec<Body> {
        let wrapper = match self.bodies.remove(&body_index) {
            Some(wrapper) => wrapper,
            None => return Vec::new(),
        };

        let (body, relations) = (wrapper.body, wrapper.relations);
        let mut bodies = vec![body];

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
    pub fn get_body_mut(&mut self, index: Id) -> Option<&mut Body> {
        self.bodies.get_mut(&index).map(|wrapper| &mut wrapper.body)
    }

    /// Gets an immutable reference to a body in the universe.
    pub fn get_body(&self, index: Id) -> Option<&Body> {
        self.bodies.get(&index).map(|wrapper| &wrapper.body)
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
