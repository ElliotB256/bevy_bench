extern crate bevy;

use bevy::app::App;
use bevy::prelude::*;

pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
pub struct Velocity {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
pub struct Timestep {
    pub dt: f64,
}

fn integrate_position(mut query: Query<(&Velocity, &mut Position)>, timestep: Res<Timestep>) {
    for (velocity, mut position) in query.iter_mut() {
        position.x = position.x + velocity.x * timestep.dt;
        position.y = position.y + velocity.y * timestep.dt;
        position.z = position.z + velocity.z * timestep.dt;
    }
}

fn spawn_atoms(mut commands: Commands) {
    // Add some atoms
    for _ in 1..10_000 {
        commands.spawn().insert_bundle((
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Velocity {
                x: -1.0,
                y: 1.0,
                z: 1.0,
            },
        ));
    }
}

fn simple_runner(mut app: App) {
    println!("Starting simulation.");
    for _ in 1..10_000 {
        app.update();
    }
    println!("Finished!");
}


fn main() {
    App::build()
        .insert_resource(Timestep { dt: 0.01 })
        .add_system(integrate_position.system())
        .add_startup_system(spawn_atoms.system())
        .set_runner(simple_runner)
        .run();
}

