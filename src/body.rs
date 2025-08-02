use keplerian_sim::Orbit;
use three_d::Srgba;

/// A struct representing a celestial body.
#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    /// The name of the celestial body.
    pub name: String,

    /// The mass of the celestial body, in kilograms.
    pub mass: f64,

    /// The radius of the celestial body, in meters.
    pub radius: f64,

    /// The color of the celestial body.
    pub color: Srgba,

    /// The orbit of the celestial body, if it is orbiting one.
    pub orbit: Option<Orbit>,
}

impl Body {
    /// Creates a new `Body` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the celestial body.
    /// * `mass` - The mass of the celestial body, in kilograms.
    /// * `radius` - The radius of the celestial body, in meters.
    /// * `orbit` - An optional orbit for the celestial body.
    ///
    /// # Returns
    ///
    /// A new `Body` instance.
    pub fn new(name: String, mass: f64, radius: f64, orbit: Option<Orbit>) -> Self {
        Self {
            name,
            mass,
            radius,
            orbit,
            color: Srgba::new_opaque(255, 255, 255),
        }
    }
}

impl Default for Body {
    /// Creates a default `Body` instance.
    ///
    /// Currently, this function returns the Earth.  
    /// However, do not rely on this behavior, as it may change in the future.
    fn default() -> Self {
        Self {
            name: "Earth".to_string(),
            mass: 5.972e24,
            radius: 6.371e6,
            orbit: None,
            color: Srgba::new_opaque(255, 255, 255),
        }
    }
}

impl From<keplerian_sim::Body> for Body {
    fn from(value: keplerian_sim::Body) -> Self {
        Self {
            name: value.name,
            mass: value.mass,
            radius: value.radius,
            orbit: value.orbit,
            color: Srgba::new_opaque(255, 255, 255),
        }
    }
}
