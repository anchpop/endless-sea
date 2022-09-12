#[cfg(test)]
mod test {
    use more_asserts::*;

    use crate::tests::helpers::*;

    use bevy::prelude::*;
    use bevy_rapier3d::prelude::*;

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
        use crate::character;
        Test {
            setup: |app| {
                app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
                    .add_plugin(character::Plugin);

                // Setup test entities
                let character_id = app
                    .world
                    .spawn()
                    .insert_bundle(SpatialBundle::default())
                    .insert_bundle(character::Bundle {
                        input: character::Input {
                            direction: Vec3::X,
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
        use crate::character;
        Test {
            setup: |app| {
                app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
                    .add_plugin(character::Plugin);

                // Setup test entities
                let character_id = app
                    .world
                    .spawn()
                    .insert_bundle(SpatialBundle::default())
                    .insert_bundle(character::Bundle {
                        input: character::Input {
                            direction: Vec3::Y,
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
}