#[cfg(test)]
mod test {
    use bevy::prelude::*;

    use crate::{terrain_generation::Island, tests::helpers::*};

    #[test]
    fn flat_gen() {
        Test {
            setup: |app| {
                let floor_size = 10.0;
                let (points, indices) = dbg!(Island::Flat.generate(
                    &default(),
                    floor_size,
                    floor_size
                ));
                let indices = bevy::render::mesh::Indices::U32(
                    indices.into_iter().flat_map(|i| i).collect(),
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
                    .map(|p| {
                        [p.color.r(), p.color.g(), p.color.b(), p.color.a()]
                    })
                    .collect::<Vec<_>>();

                let mut mesh = Mesh::new(
                    bevy::render::mesh::PrimitiveTopology::TriangleList,
                );
                mesh.set_indices(Some(indices));
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

                if let Some(mut meshes) =
                    app.world.get_resource_mut::<Assets<Mesh>>()
                {
                    let mesh = meshes.add(mesh);
                    if let Some(mut materials) =
                        app.world.get_resource_mut::<Assets<StandardMaterial>>()
                    {
                        let material =
                            materials.add(StandardMaterial::default());

                        app.world
                            .spawn()
                            .insert_bundle(PbrBundle {
                                mesh,
                                material,
                                transform: Transform::from_xyz(
                                    -(floor_size / 2.0),
                                    0.0,
                                    -(floor_size / 2.0),
                                ),
                                ..Default::default()
                            })
                            .insert(Name::new("Bumpy Floor"));
                    }
                }
            },
            setup_graphics: default_setup_graphics,
            frames: 1,
            check: |_app, ()| {},
        }
        .run()
    }
}
