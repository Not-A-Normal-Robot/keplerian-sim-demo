pub(crate) mod body;
mod presets;
pub(crate) mod universe;

macro_rules! declare_universe {
    {
        $root_name:ident $( { } )?
    } => {{
        let mut universe = universe::Universe::default();

        universe.add_body(presets::$root_name(None), None).unwrap();

        universe
    }};
    {
        $root_name:ident $( { $($inner:tt)* } )?
    } => {{
        let mut universe = universe::Universe::default();
        let g = universe.get_gravitational_constant();

        ::pastey::paste!{
            let [<$root_name _body>] = presets::$root_name(None);
            let [<$root_name _mu>] = [<$root_name _body>].mass * g;
            let [<$root_name _id>] =
                universe.add_body([<$root_name _body>], None).unwrap();

            declare_universe!(
                @inner(universe, g),
                ([<$root_name _mu>], [<$root_name _id>]),
                $($($inner)*)?
            );
        }

        universe
    }};
    (
        @inner($universe:ident, $g:ident),
        ($parent_mu:ident, $parent_id:ident),
        $this:ident $( { } )? $(, $($tail:tt)*)?
    ) => {
        $universe.add_body(presets::$this(Some($parent_mu)), Some($parent_id)).unwrap();

        declare_universe!(
            @inner($universe, $g),
            ($parent_mu, $parent_id),
            $($($tail)*)?
        );
    };
    (
        @inner($universe:ident, $g:ident),
        ($parent_mu:ident, $parent_id:ident),
    ) => {};
    (
        @inner($universe:ident, $g:ident),
        ($parent_mu:ident, $parent_id:ident),
        $this:ident $( { $($children:tt)* } )? $(, $($tail:tt)*)?
    ) => {
        ::pastey::paste!{
            let [<$this _body>] = presets::$this(Some($parent_mu));
            let [<$this _mu>] = [<$this _body>].mass * $g;
            let [<$this _id>] =
                $universe.add_body([<$this _body>], Some($parent_id)).unwrap();

            declare_universe!(
                @inner($universe, $g),
                ([<$this _mu>], [<$this _id>]),
                $($($children)*)?
            );
            declare_universe!(
                @inner($universe, $g),
                ($parent_mu, $parent_id),
                $($($tail)*)?
            );
        }
    };
}

pub(crate) fn create_universe() -> universe::Universe {
    declare_universe! {
        the_sun {
            mercury,
            venus,
            earth {
                luna,
            },
            mars {
                phobos, deimos,
            },
            ceres,
            vesta,
            jupiter {
                // io, europa, ganymede, callisto
            },
            saturn {
                // titan, enceladus, mimas, iapetus
            },
            uranus {
                // tiania, oberon
            },
            neptune {
                // triton, proteus, nereid
            },
            pluto {
                charon,
            },
            eris {
                dysnomia,
            },
            quaoar {
                weywot,
            },
            haumea,
            makemake,
            sedna,
            leleakuhonua,
        }
    }
}
