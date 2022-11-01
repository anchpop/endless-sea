#[cfg(test)]
mod test {
    use bevy::{prelude::*, sprite::Rect};
    use bevy_rapier3d::prelude::*;

    use crate::{
        terrain_generation::{Generation, Island},
        tests::helpers::*,
    };

    #[test]
    fn flat_gen() {
        Test {
            setup: |app| {
                show(
                    app,
                    Island::Flat,
                    Generation {
                        vertex_density: 1.0,
                    },
                    10.0,
                );
            },
            setup_graphics: default_setup_graphics,
            frames: 1,
            check: |_app, ()| {},
        }
        .run()
    }

    #[test]
    fn simplex_gen() {
        Test {
            setup: |app| {
                show(
                    app,
                    Island::Simplex(0),
                    Generation {
                        vertex_density: 1.0,
                    },
                    10.0,
                );
            },
            setup_graphics: default_setup_graphics,
            frames: 1,
            check: |_app, ()| {},
        }
        .run()
    }

    fn show(
        app: &mut App,
        island: Island,
        generation_type: Generation,
        size: f32,
    ) {
        let (points, indices) = island.generate(
            &generation_type,
            Rect {
                min: Vec2::new(-size / 2.0, -size / 2.0),
                max: Vec2::new(size / 2.0, size / 2.0),
            },
        );

        let mesh = {
            let indices = bevy::render::mesh::Indices::U32(
                indices.iter().cloned().flat_map(|i| i).collect(),
            );
            let positions = points
                .iter()
                .map(|p| [p.position.x, p.position.y, p.position.z])
                .collect::<Vec<_>>();
            let normals = points
                .iter()
                .map(|p| [p.normal.x, p.normal.y, p.normal.z])
                .collect::<Vec<_>>();
            let colors = points
                .iter()
                .map(|p| [p.color.r(), p.color.g(), p.color.b(), p.color.a()])
                .collect::<Vec<_>>();

            let mut mesh =
                Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
            mesh.set_indices(Some(indices));
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            mesh
        };

        if let Some(mut meshes) = app.world.get_resource_mut::<Assets<Mesh>>() {
            let mesh = meshes.add(mesh);
            if let Some(mut materials) =
                app.world.get_resource_mut::<Assets<StandardMaterial>>()
            {
                let material = materials.add(StandardMaterial::default());

                app.world
                    .spawn()
                    .insert_bundle(PbrBundle {
                        mesh,
                        material,
                        transform: Transform::from_xyz(0.0, 0.0, 0.0),
                        ..Default::default()
                    })
                    .insert(Collider::trimesh(
                        points.iter().map(|p| p.position).collect(),
                        indices,
                    ))
                    .insert(Name::new("generated mesh"));
            }
        }
    }
}
