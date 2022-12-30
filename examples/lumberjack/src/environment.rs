use bevy::prelude::*;

#[derive(Component)]
pub struct Axe;

pub fn create_axes_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let axe_transforms = vec![
        Transform::from_xyz(5., 0., -20.),
        Transform::from_xyz(20., 0., -40.),
    ];

    for transform in axe_transforms {
        commands.spawn_empty().insert(Axe).insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1. })),
            material: materials.add(Color::BLUE.into()),
            transform,
            ..Default::default()
        });
    }
}

#[derive(Component)]
pub struct Tree;

pub fn create_trees_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tree_transforms = vec![
        Transform::from_xyz(35., 0., 35.),
        Transform::from_xyz(-35., 0., 35.),
        Transform::from_xyz(35., 0., -35.),
        Transform::from_xyz(-35., 0., -35.),
    ];

    for transform in tree_transforms {
        commands.spawn_empty().insert(Tree).insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(2., 24., 2.))),
            material: materials.add(Color::BEIGE.into()),
            transform,
            ..Default::default()
        });
    }
}
