use bevy::prelude::*;

#[derive(Component)]
pub struct Navigation {
    pub navigator: Entity,
    pub target: Entity,
    pub speed: f32,
    pub is_done: bool,
}

pub fn navigation_system(
    mut navigation_query: Query<&mut Navigation>,
    mut transforms_query: Query<&mut Transform>,
    time: Res<Time>,
) {
    for mut nav in navigation_query.iter_mut() {
        if nav.is_done {
            continue;
        }

        let navigator_position = transforms_query.get(nav.navigator).unwrap().translation;
        let target_position = transforms_query.get(nav.target).unwrap().translation;

        let delta_to_target = target_position - navigator_position;

        if delta_to_target.length() < 1. {
            nav.is_done = true;
            continue;
        } else {
            let movement_delta = nav.speed * time.delta_seconds() * delta_to_target.normalize();
            transforms_query.get_mut(nav.navigator).unwrap().translation += movement_delta;
        }
    }
}
