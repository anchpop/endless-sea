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

    fn spawn_floor_beneath_capsule(app: &mut App, capsule_id: Entity) {
        let transform = *app.world.get::<Transform>(capsule_id).unwrap();
        let collider = app.world.get::<Collider>(capsule_id).unwrap().clone();
        let capsule = collider.as_capsule().unwrap();
        app.world
            .spawn()
            .insert(Collider::cuboid(0.5, 0.5, 0.5))
            .insert_bundle(TransformBundle::from(Transform {
                translation: Vec3::ZERO
                    - transform.translation
                    - Vec3::Y * capsule.height(),
                scale: Vec3::new(10.0, 1.0, 10.0),
                ..Default::default()
            }))
            .insert(Name::new("floor"));
    }

    #[test]
    fn character_moves_horizontally() {
        Test {
            setup: |app| {
                app.add_plugin(character::Plugin);

                // Setup test entities
                let character_id = app
                    .world
                    .spawn()
                    .insert_bundle(SpatialBundle::default())
                    .insert_bundle(character::Bundle {
                        input: character::Input {
                            movement_direction: Vec3::X,
                            ..character::Input::default()
                        },
                        ..character::Bundle::default()
                    })
                    .id();
                spawn_floor_beneath_capsule(app, character_id);
                character_id
            },
            setup_graphics: default_setup_graphics,
            frames: 10,
            check: |app, character_id| {
                let character =
                    app.world.get::<Transform>(character_id).unwrap();
                assert_gt!(character.translation.x, 0.0);
            },
        }
        .run()
    }

    #[test]
    fn character_doesnt_move_vertically() {
        Test {
            setup: |app| {
                app.add_plugin(character::Plugin);

                // Setup test entities
                let character_id = app
                    .world
                    .spawn()
                    .insert_bundle(SpatialBundle::default())
                    .insert_bundle(character::Bundle {
                        input: character::Input {
                            movement_direction: Vec3::Y,
                            ..character::Input::default()
                        },
                        ..character::Bundle::default()
                    })
                    .insert(Name::new("character"))
                    .id();
                let initial_character_translation = app
                    .world
                    .get::<Transform>(character_id)
                    .unwrap()
                    .translation;
                spawn_floor_beneath_capsule(app, character_id);
                (character_id, initial_character_translation)
            },
            setup_graphics: default_setup_graphics,
            frames: 100,
            check: |app, (character_id, initial_character_translation)| {
                let character =
                    app.world.get::<Transform>(character_id).unwrap();
                assert!(
                    (character.translation.y - initial_character_translation.y)
                        .abs()
                        < 0.01,
                );
            },
        }
        .run()
    }

    #[test]
    fn create_world() {
        Test {
            setup: |app| {
                // Setup test entities
                let character_id = app
                    .world
                    .spawn()
                    .insert_bundle(SpatialBundle::default())
                    .insert(Name::new("character"))
                    .id();
                let initial_character_translation = app
                    .world
                    .get::<Transform>(character_id)
                    .unwrap()
                    .translation;
                (character_id, initial_character_translation)
            },
            setup_graphics: default_setup_graphics,
            frames: 100,
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
                app.add_plugin(object::Plugin).add_plugin(character::Plugin);

                // Setup test entities
                let character_id = app
                    .world
                    .spawn()
                    .insert_bundle(SpatialBundle::default())
                    .insert_bundle(character::Bundle {
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
            frames: 10,
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
                app.add_plugin(character::Plugin).add_plugin(object::Plugin);

                // Setup test entities
                let character_id = app
                    .world
                    .spawn()
                    .insert_bundle(SpatialBundle::default())
                    .insert_bundle(character::Bundle {
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
                    })
                    .insert_bundle(player::Bundle::default())
                    .id();

                let object_id = app
                    .world
                    .spawn()
                    .insert(RigidBody::Dynamic)
                    .insert(Collider::cuboid(0.5, 0.5, 0.5))
                    .insert_bundle(object::Bundle::default())
                    .insert_bundle(SpatialBundle {
                        transform: Transform::from_xyz(2.0, 0.001, 0.0),
                        ..default()
                    })
                    .insert(Name::new("Obstacle"))
                    .id();

                spawn_floor_beneath_capsule(app, character_id);
                object_id
            },
            setup_graphics: default_setup_graphics,
            frames: 1000,
            check: |app, object_id| {
                app.world
                    .get::<Transform>(object_id)
                    .expect("object should still exist");
                let health =
                    app.world.get::<object::Health>(object_id).unwrap().clone();
                assert_lt!(
                    health.current,
                    health.max,
                    "Character should have taken damage"
                );
            },
        }
        .run()
    }
}
