#![cfg(test)]
#![allow(missing_docs)]
#![allow(clippy::pedantic)]

use bevy::MinimalPlugins;
use bevy::app::FixedPostUpdate;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::{App, Component, Fixed, Time, World};
use bevy::tasks::tick_global_task_pools_on_main_thread;
use std::time::Duration;
use utils::{AsyncComponent, CorePlugin};

#[derive(Component)]
struct FooComponent {
    pub bar: &'static str,
}

#[test]
fn spawn_new_async() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CorePlugin));

    app.world_mut()
        .spawn(AsyncComponent::new_async(async |sender| {
            let mut queue = CommandQueue::default();

            queue.push(|world: &mut World| {
                world.spawn(FooComponent { bar: "baz" });
            });

            sender.send(queue).unwrap();
            Ok(())
        }));

    app.update(); // execute spawn of AsyncComponent
    tick_global_task_pools_on_main_thread(); // run background runners
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_secs(2)); // "advance" game time
    app.world_mut().run_schedule(FixedPostUpdate); // force FixedPostUpdate schedule to run
    app.update(); // run any commands that have been appended by FixedPostUpdate

    let foo: Vec<&FooComponent> = app
        .world_mut()
        .query::<&FooComponent>()
        .iter(app.world())
        .collect();
    assert_eq!(foo.len(), 1, "A FooComponent should have been spawned");
    assert_eq!(foo[0].bar, "baz", "FooComponent should have correct value");

    let component = app
        .world_mut()
        .query::<&AsyncComponent>()
        .single(app.world());
    assert!(
        component.is_err(),
        "There should no longer be an AsyncComponent"
    );
}

#[test]
fn spawn_new_compute() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CorePlugin));

    app.world_mut()
        .spawn(AsyncComponent::new_compute(async |sender| {
            let mut queue = CommandQueue::default();

            queue.push(|world: &mut World| {
                world.spawn(FooComponent { bar: "bazz" });
            });

            sender.send(queue).unwrap();
            Ok(())
        }));

    app.update(); // execute spawn of AsyncComponent
    tick_global_task_pools_on_main_thread(); // Run background runners
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_secs(2)); // "Advance" game time
    app.world_mut().run_schedule(FixedPostUpdate); // Force FixedPostUpdate schedule to run
    app.update(); // Run any commands that have been appended by FixedPostUpdate

    let foo: Vec<&FooComponent> = app
        .world_mut()
        .query::<&FooComponent>()
        .iter(app.world())
        .collect();
    assert_eq!(foo.len(), 1, "A FooComponent should have been spawned");
    assert_eq!(foo[0].bar, "bazz", "FooComponent should have correct value");

    let component = app
        .world_mut()
        .query::<&AsyncComponent>()
        .single(app.world());
    assert!(
        component.is_err(),
        "There should no longer be an AsyncComponent"
    );
}

#[test]
fn spawn_new_io() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CorePlugin));

    app.world_mut()
        .spawn(AsyncComponent::new_io(async |sender| {
            let mut queue = CommandQueue::default();

            queue.push(|world: &mut World| {
                world.spawn(FooComponent { bar: "bazzz" });
            });

            sender.send(queue).unwrap();
            Ok(())
        }));

    app.update(); // execute spawn of AsyncComponent
    tick_global_task_pools_on_main_thread(); // Run background runners
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_secs(2)); // "Advance" game time
    app.world_mut().run_schedule(FixedPostUpdate); // Force FixedPostUpdate schedule to run
    app.update(); // Run any commands that have been appended by FixedPostUpdate

    let foo: Vec<&FooComponent> = app
        .world_mut()
        .query::<&FooComponent>()
        .iter(app.world())
        .collect();
    assert_eq!(foo.len(), 1, "A FooComponent should have been spawned");
    assert_eq!(
        foo[0].bar, "bazzz",
        "FooComponent should have correct value"
    );

    let component = app
        .world_mut()
        .query::<&AsyncComponent>()
        .single(app.world());
    assert!(
        component.is_err(),
        "There should no longer be an AsyncComponent"
    );
}
