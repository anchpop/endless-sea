// Example usage: cargo test character_moves_horizontally -- --test-threads=1
// --nocapture

#[cfg(test)]
mod test {
    use bevy::prelude::*;
    use bevy_rapier3d::prelude::*;
    use more_asserts::*;

    use crate::{
        character,
        item::{self, HeldItem},
        object, player,
        tests::helpers::*,
    };

    #[test]
    fn create_world() {
        Test {
            setup: |app| {
                // Setup test entities
                let character_id = app
                    .world
                    .spawn((SpatialBundle::default(), Name::new("character")))
                    .id();
                let initial_character_translation = app
                    .world
                    .get::<Transform>(character_id)
                    .unwrap()
                    .translation;
                (character_id, initial_character_translation)
            },
            setup_graphics: default_setup_graphics,
            frames: 1,
            check: |app, (character_id, initial_character_translation)| {
                let character =
                    app.world.get::<Transform>(character_id).unwrap();
                assert!(
                    character.translation.y == initial_character_translation.y
                );
            },
        }
        .run()
    }

    #[test]
    fn character_dies() {
        Test {
            setup: |app| {
                app.add_plugins((object::Plugin, character::Plugin));

                // Setup test entities
                let character_id = app
                    .world
                    .spawn(character::Bundle {
                        health: object::Health {
                            current: 0.0,
                            ..object::Health::default()
                        },
                        ..character::Bundle::default()
                    })
                    .id();
                spawn_floor_beneath_capsule(app, character_id);
                character_id
            },
            setup_graphics: default_setup_graphics,
            frames: 1,
            check: |app, character_id| {
                assert!(
                    app.world.get::<Transform>(character_id).is_none(),
                    "Character should be despawned"
                );
            },
        }
        .run()
    }

    #[test]
    fn sword_attack() {
        Test {
            setup: |app| {
                app.add_plugins((character::Plugin, object::Plugin));

                // Setup test entities
                let character_id = app
                    .world
                    .spawn((
                        character::Bundle {
                            inventory: character::Inventory {
                                hand: Some(HeldItem::new(item::Item::Sword)),
                                ..character::Inventory::default()
                            },
                            input: character::Input {
                                looking_direction: Vec3::X,
                                attack: Some(character::AttackState::Primary),
                                ..character::Input::default()
                            },
                            ..character::Bundle::default()
                        },
                        player::Bundle::default(),
                    ))
                    .id();

                let object_id = app
                    .world
                    .spawn((
                        RigidBody::Dynamic,
                        Collider::cuboid(0.5, 0.5, 0.5),
                        object::Bundle::default(),
                        SpatialBundle {
                            transform: Transform::from_xyz(1.5, 0.001, 0.0),
                            ..default()
                        },
                        Name::new("Obstacle"),
                    ))
                    .id();

                spawn_floor_beneath_capsule(app, character_id);
                object_id
            },
            setup_graphics: default_setup_graphics,
            // Rapier raycasts do nothing the first frame, so we have to wait
            // enough frames for the cooldown to expire so we attempt to hit
            // again.
            frames: (item::Item::Sword.cooldown().as_secs_f32() * TEST_FPS)
                .ceil() as u64
                + 5,
            check: |app, object_id| {
                app.world
                    .get::<Transform>(object_id)
                    .expect("object should still exist");
                let health =
                    app.world.get::<object::Health>(object_id).unwrap().clone();
                assert_lt!(
                    health.current,
                    health.max,
                    "object should have taken damage"
                );
            },
        }
        .run()
    }
}
