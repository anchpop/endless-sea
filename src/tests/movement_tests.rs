// Example usage: cargo test character_moves_horizontally -- --test-threads=1
// --nocapture

#[cfg(test)]
mod test {
    use bevy::prelude::*;
    use bevy_rapier3d::prelude::*;
    use more_asserts::*;

    use crate::{character, player, tests::helpers::*};

    #[test]
    fn character_moves_horizontally() {
        Test {
            setup: |app| {
                app.add_plugin(character::Plugin);

                // Setup test entities
                let character_id = app
                    .world
                    .spawn((
                        SpatialBundle::default(),
                        character::Bundle {
                            input: character::Input {
                                movement_direction: Vec3::X,
                                ..character::Input::default()
                            },
                            ..character::Bundle::default()
                        },
                        player::Bundle::default(),
                    ))
                    .id();
                spawn_floor_beneath_capsule(app, character_id);
                character_id
            },
            setup_graphics: default_setup_graphics,
            // Rapier does nothing the first frame, so we have to use 2 frames
            // here
            frames: 3,
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
                    .spawn((
                        SpatialBundle::default(),
                        character::Bundle {
                            input: character::Input {
                                movement_direction: Vec3::Y,
                                ..character::Input::default()
                            },
                            ..character::Bundle::default()
                        },
                        Name::new("character"),
                    ))
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
            frames: 1,
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
    fn character_doesnt_exceed_max_speed() {
        const MAX_SPEED: f32 = 1.0;
        Test {
            setup: |app| {
                app.add_plugin(character::Plugin);

                // Setup test entities
                let character_id = app
                    .world
                    .spawn((
                        SpatialBundle::default(),
                        character::Bundle {
                            input: character::Input {
                                movement_direction: Vec3::X,
                                ..character::Input::default()
                            },
                            movement_properties:
                                character::MovementProperties {
                                    max_speed: MAX_SPEED,
                                    ..character::MovementProperties::default()
                                },
                            ..character::Bundle::default()
                        },
                    ))
                    .id();
                spawn_floor_beneath_capsule(app, character_id);
                character_id
            },
            setup_graphics: default_setup_graphics,
            // wait plenty of frames for the player to pick up speed
            frames: 30,
            check: |app, character_id| {
                let velocity = app.world.get::<Velocity>(character_id).unwrap();
                assert!(
                    velocity.linvel.length() - 0.05 <= MAX_SPEED,
                    "Player's speed was {} while the max speed was {MAX_SPEED}",
                    velocity.linvel.length()
                );
            },
        }
        .run()
    }
}
