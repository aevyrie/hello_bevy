use bevy::prelude::*;

fn main() {
    App::build()
        .add_default_plugins()
        .add_startup_system(add_people.system())
        .add_system(hello_world.system())
        .add_system(greet_people.system())
        .run();
}

fn hello_world() {
    println!("Hello world!");
}

struct Person;

struct Name(String);

fn add_people(mut commands: Commands) {
    commands
        .spawn((Person, Name("Bob Belcher".to_string())))
        .spawn((Person, Name("Linda Belcher".to_string())))
        .spawn((Person, Name("Tina Belcher".to_string())));
}

fn greet_people(person: &Person, name: &Name) {
    println!("Hello {}!", name.0);
}