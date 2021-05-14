//! A minimal specs example.
//!
//! Performs a velocity-verlet integration of particles in a harmonic trap.

extern crate specs;
use specs::prelude::*;

extern crate nalgebra;
use nalgebra::Vector3;

pub struct Position(Vector3<f64>);
impl Component for Position {
    type Storage = VecStorage<Self>;
}

pub struct Velocity(Vector3<f64>);
impl Component for Velocity {
    type Storage = VecStorage<Self>;
}

#[derive(Copy, Clone)]
pub struct Force(Vector3<f64>);
impl Component for Force {
    type Storage = VecStorage<Self>;
}

pub struct OldForce(Force);
impl Component for OldForce {
    type Storage = VecStorage<Self>;
}

pub struct Mass(f64);
impl Component for Mass {
    type Storage = VecStorage<Self>;
}

pub struct Timestep {
    pub dt: f64,
}

pub struct IntegratePositionSystem;
impl<'a> System<'a> for IntegratePositionSystem {
    type SystemData = (
        ReadStorage<'a, Velocity>,
        ReadStorage<'a, Mass>,
        ReadStorage<'a, Force>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, OldForce>,
        ReadExpect<'a, Timestep>
    );

    fn run(&mut self, (vel, mass, force, mut pos, mut old_force, timestep): Self::SystemData) {
        use rayon::prelude::*;
        
        let dt = timestep.dt;

        (&vel, &mass, &force, &mut pos, &mut old_force)
            .par_join()
            .for_each(|(vel, mass, force, mut pos, mut old_force)| {
                pos.0 = pos.0 + vel.0 * dt + force.0 / (mass.0) / 2.0 * dt * dt;
                old_force.0 = *force;
            });
    }
}

pub struct IntegrateVelocitySystem;
impl<'a> System<'a> for IntegrateVelocitySystem {
    type SystemData = (
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Force>,
        ReadStorage<'a, OldForce>,
        ReadStorage<'a, Mass>,
        ReadExpect<'a, Timestep>
    );

    fn run(&mut self, (mut vel, force, old_force, mass, timestep): Self::SystemData) {
        use rayon::prelude::*;
        
        let dt = timestep.dt;

        (&mut vel, &force, &old_force, &mass).par_join().for_each(
            |(mut vel, force, old_force, mass)| {
                vel.0 = vel.0 + (force.0 + old_force.0 .0) / (mass.0) / 2.0 * dt;
            },
        );
    }
}

pub struct HarmonicForceSystem;
impl<'a> System<'a> for HarmonicForceSystem {
    type SystemData = (WriteStorage<'a, Force>, ReadStorage<'a, Position>);

    fn run(&mut self, (mut force, pos): Self::SystemData) {
        use rayon::prelude::*;

        (&mut force, &pos).par_join().for_each(|(mut force, pos)| {
            force.0 = -pos.0;
        });
    }
}

fn main() {
    let mut world = World::new();
    let mut builder = DispatcherBuilder::new();

    builder.add(IntegratePositionSystem, "integrate_position", &[]);
    builder.add(HarmonicForceSystem, "harmonic", &["integrate_position"]);
    builder.add(IntegrateVelocitySystem, "integrate_velocity", &["harmonic"]);

    //// Configure pool if you like:
    ////
    // let pool = rayon::ThreadPoolBuilder::new()
    //     .num_threads(6)
    //     .build()
    //     .unwrap();
    // builder.add_pool(::std::sync::Arc::new(pool));

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world);

    world.insert(Timestep { dt: 1.0 });

    for _ in 0..100_000 {
        world
            .create_entity()
            .with(Position {
                0: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(Velocity {
                0: Vector3::new(0.2, 0.5, 1.0),
            })
            .with(Mass { 0: 1.0 })
            .with(Force {
                0: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(OldForce {
                0: Force {
                    0: Vector3::new(0.0, 0.0, 0.0),
                },
            })
            .build();
    }

    println!("Starting simulation.");
    for _ in 0..1_000 {
        dispatcher.dispatch(&mut world);
        world.maintain();
    }
    println!("Finished!");
}
