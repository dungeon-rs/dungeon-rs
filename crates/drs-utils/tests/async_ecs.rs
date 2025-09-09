#![cfg(test)]
#![allow(missing_docs)]
#![allow(clippy::pedantic)]

use bevy::MinimalPlugins;
use bevy::app::FixedPostUpdate;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::{App, BevyError, Component, Event, Events, Fixed, Time, World};
use bevy::tasks::tick_global_task_pools_on_main_thread;
use drs_utils::{AsyncComponent, CorePlugin, command_queue, report_progress, send_command};
use std::time::Duration;

#[derive(Component)]
struct FooComponent {
    pub bar: &'static str,
}

#[derive(Event)]
struct FooEvent {
    pub bar: String,
}

/// Advance the world
fn advance_world(app: &mut App) {
    app.update();
    tick_global_task_pools_on_main_thread();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_secs(2));
    app.world_mut().run_schedule(FixedPostUpdate);
    app.update();
}

#[test]
fn spawn_new_async() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CorePlugin));

    app.world_mut().spawn(AsyncComponent::new_async(
        async |sender| {
            let mut queue = CommandQueue::default();

            queue.push(|world: &mut World| {
                world.spawn(FooComponent { bar: "baz" });
            });

            sender.send(queue).unwrap();
            Ok(())
        },
        |_, _| {
            panic!("Should not fail");
        },
    ));

    advance_world(&mut app);

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
fn calls_error_on_failure() {
    let mut app = App::new();
    app.add_event::<FooEvent>();
    app.add_plugins((MinimalPlugins, CorePlugin));

    app.world_mut().spawn(AsyncComponent::new_async(
        async |_sender| -> Result<(), BevyError> { Err(BevyError::from("this went wrong")) },
        |error, sender| {
            let mut queue = CommandQueue::default();
            queue.push(move |world: &mut World| {
                world.send_event(FooEvent {
                    bar: error.to_string(),
                });
            });
            sender.send(queue).unwrap();
        },
    ));

    advance_world(&mut app);

    let component = app
        .world_mut()
        .query::<&AsyncComponent>()
        .single(app.world());
    assert!(
        component.is_err(),
        "There should no longer be an AsyncComponent"
    );

    let events = app.world_mut().resource_mut::<Events<FooEvent>>();
    let mut cursor = events.get_cursor();
    let event = cursor.read(&events).next();
    assert!(
        event.is_some(),
        "The error handler should have spawned a FooEvent"
    );
    assert!(
        event.unwrap().bar.starts_with("this went wrong"),
        "The bar should have correct value"
    );
}

#[test]
fn spawn_new_compute() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CorePlugin));

    app.world_mut().spawn(AsyncComponent::new_compute(
        async |sender| {
            let mut queue = CommandQueue::default();

            queue.push(|world: &mut World| {
                world.spawn(FooComponent { bar: "bazz" });
            });

            sender.send(queue).unwrap();
            Ok(())
        },
        |_, _| {
            panic!("Should not fail");
        },
    ));

    advance_world(&mut app);

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

    app.world_mut().spawn(AsyncComponent::new_io(
        async |sender| {
            let mut queue = CommandQueue::default();

            queue.push(|world: &mut World| {
                world.spawn(FooComponent { bar: "bazzz" });
            });

            sender.send(queue).unwrap();
            Ok(())
        },
        |_, _| {
            panic!("Should not fail");
        },
    ));

    advance_world(&mut app);

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

#[test]
fn test_report_progress_success() {
    let (sender, receiver) = command_queue();

    let result = report_progress(
        &sender,
        FooEvent {
            bar: "test progress".to_string(),
        },
    );

    assert!(result.is_ok(), "report_progress should succeed");

    let mut world = World::new();
    world.init_resource::<Events<FooEvent>>();

    let mut queue = receiver.try_recv().expect("Should receive command queue");
    let mut cmd_queue = CommandQueue::default();
    cmd_queue.append(&mut queue);
    cmd_queue.apply(&mut world);

    let events = world.resource::<Events<FooEvent>>();
    let mut cursor = events.get_cursor();
    let event = cursor.read(events).next();

    assert!(event.is_some(), "Event should have been sent");
    assert_eq!(event.unwrap().bar, "test progress");
}

#[test]
fn test_report_progress_sender_disconnected() {
    let (sender, receiver) = command_queue();
    drop(receiver);

    let result = report_progress(
        &sender,
        FooEvent {
            bar: "test".to_string(),
        },
    );

    assert!(
        result.is_err(),
        "report_progress should fail when receiver is dropped"
    );
}

#[test]
fn test_send_command_success() {
    let (sender, receiver) = command_queue();

    let result = send_command(&sender, |world: &mut World| {
        world.spawn(FooComponent {
            bar: "test_command",
        });
    });

    assert!(result.is_ok(), "send_command should succeed");

    let mut world = World::new();

    let mut queue = receiver.try_recv().expect("Should receive command queue");
    let mut cmd_queue = CommandQueue::default();
    cmd_queue.append(&mut queue);
    cmd_queue.apply(&mut world);

    let components: Vec<&FooComponent> = world.query::<&FooComponent>().iter(&world).collect();
    assert_eq!(components.len(), 1, "Component should have been spawned");
    assert_eq!(components[0].bar, "test_command");
}

#[test]
fn test_send_command_sender_disconnected() {
    let (sender, receiver) = command_queue();
    drop(receiver);

    let result = send_command(&sender, |world: &mut World| {
        world.spawn(FooComponent { bar: "test" });
    });

    assert!(
        result.is_err(),
        "send_command should fail when receiver is dropped"
    );
}

#[test]
fn test_send_command_multiple_operations() {
    let (sender, receiver) = command_queue();

    send_command(&sender, |world: &mut World| {
        world.spawn(FooComponent { bar: "first" });
    })
    .expect("First command should succeed");

    send_command(&sender, |world: &mut World| {
        world.spawn(FooComponent { bar: "second" });
    })
    .expect("Second command should succeed");

    let mut world = World::new();

    while let Ok(mut queue) = receiver.try_recv() {
        let mut cmd_queue = CommandQueue::default();
        cmd_queue.append(&mut queue);
        cmd_queue.apply(&mut world);
    }

    let components: Vec<&FooComponent> = world.query::<&FooComponent>().iter(&world).collect();
    assert_eq!(
        components.len(),
        2,
        "Both components should have been spawned"
    );

    let bars: Vec<&str> = components.iter().map(|c| c.bar).collect();
    assert!(bars.contains(&"first"), "Should contain first component");
    assert!(bars.contains(&"second"), "Should contain second component");
}
