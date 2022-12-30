use bevy::prelude::*;

#[derive(Component)]
pub struct Axe;

pub fn create_axes_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_empty().insert(Axe).insert(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 16. })),
        material: materials.add(Color::BLUE.into()),
        transform: Transform::from_xyz(100., 0., 100.),
        ..Default::default()
    });
}

#[derive(Component)]
pub struct Tree;

pub fn create_trees_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tree_transforms = vec![
        Transform::from_xyz(-100., 0., 100.),
        Transform::from_xyz(100., 0., 200.),
        Transform::from_xyz(-400., 0., 250.),
        Transform::from_xyz(300., 0., 300.),
    ];

    for transform in tree_transforms {
        commands.spawn_empty().insert(Tree).insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(24., 128., 24.))),
            material: materials.add(Color::BEIGE.into()),
            transform,
            ..Default::default()
        });
    }
}
