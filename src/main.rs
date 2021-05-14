extern crate bevy;

use bevy::app::App;
use bevy::{prelude::*, tasks::prelude::*};

extern crate nalgebra;
use nalgebra::Vector3;

pub struct Position(Vector3<f64>);
pub struct Velocity(Vector3<f64>);
#[derive(Copy, Clone)]
pub struct Force(Vector3<f64>);
pub struct OldForce(Force);
pub struct Mass(f64);

pub struct Timestep {
    pub dt: f64,
}

fn integrate_position(
    pool: Res<ComputeTaskPool>,
    mut query: Query<(&Velocity, &Mass, &Force, &mut Position, &mut OldForce)>,
    timestep: Res<Timestep>,
) {
    let dt = timestep.dt;
    query.par_for_each_mut(&pool, 32, |(vel, mass, force, mut pos, mut old_force)| {
        pos.0 = pos.0 + vel.0 * dt + force.0 / (mass.0) / 2.0 * dt * dt;
        old_force.0 = *force;
    });
}

fn integrate_velocity(
    pool: Res<ComputeTaskPool>,
    mut query: Query<(&mut Velocity, &Force, &OldForce, &Mass)>,
    timestep: Res<Timestep>,
) {
    let dt = timestep.dt;
    query.par_for_each_mut(&pool, 32, |(mut vel, force, old_force, mass)| {
        vel.0 = vel.0 + (force.0 + old_force.0 .0) / (mass.0) / 2.0 * dt;
    });
}

fn harmonic_trap(pool: Res<ComputeTaskPool>, mut query: Query<(&mut Force, &Position)>) {
    query.par_for_each_mut(&pool, 32, |(mut force, pos)| {
        force.0 = -pos.0;
    });
}

fn spawn_atoms(mut commands: Commands) {
    // Add some atoms
    for _ in 1..100_000 {
        commands.spawn().insert_bundle((
            Position {
                0: Vector3::new(0.0, 0.0, 0.0),
            },
            Velocity {
                0: Vector3::new(0.2, 0.5, 1.0),
            },
            Mass { 0: 1.0 },
            Force {
                0: Vector3::new(0.0, 0.0, 0.0),
            },
            OldForce {
                0: Force {
                    0: Vector3::new(0.0, 0.0, 0.0),
                },
            },
        ));
    }
}

fn simple_runner(mut app: App) {
    println!("Starting simulation.");
    for _ in 1..1_000 {
        app.update();
    }
    println!("Finished!");
}

fn main() {
    let mut pool = DefaultTaskPoolOptions::default();
    pool.io = bevy::core::task_pool_options::TaskPoolThreadAssignmentPolicy {
        min_threads: 0,
        max_threads: 0,
        percent: 0.0,
    };
    pool.async_compute = bevy::core::task_pool_options::TaskPoolThreadAssignmentPolicy {
        min_threads: 0,
        max_threads: 12,
        percent: 1.0,
    };
    pool.compute = bevy::core::task_pool_options::TaskPoolThreadAssignmentPolicy {
        min_threads: 1,
        max_threads: 1,
        percent: 0.1,
    };

    App::build()
        .insert_resource(Timestep { dt: 1.0 })
        .add_system(integrate_position.system())
        .add_system(harmonic_trap.system())
        .add_system(integrate_velocity.system())
        .add_startup_system(spawn_atoms.system())
        .set_runner(simple_runner)
        .insert_resource(pool)
        .run();
}
