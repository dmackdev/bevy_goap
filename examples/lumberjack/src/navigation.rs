use bevy::prelude::*;

#[derive(Component)]
pub struct Navigation {
    pub navigator: Entity,
    pub target: Entity,
    pub speed: f32,
    pub is_done: bool,
    pub is_err: bool,
}

pub fn navigation_system(
    mut navigation_query: Query<&mut Navigation>,
    mut transforms_query: Query<&mut Transform>,
    time: Res<Time>,
) {
    for mut nav in navigation_query.iter_mut() {
        if nav.is_done || nav.is_err {
            continue;
        }

        if let (Ok(navigator_transform), Ok(target_transform)) = (
            transforms_query.get(nav.navigator),
            transforms_query.get(nav.target),
        ) {
            let navigator_position = navigator_transform.translation;
            let target_position = target_transform.translation;

            let delta_to_target = target_position - navigator_position;

            if delta_to_target.length() < 1. {
                nav.is_done = true;
                continue;
            } else {
                let movement_delta = nav.speed * time.delta_seconds() * delta_to_target.normalize();
                transforms_query.get_mut(nav.navigator).unwrap().translation += movement_delta;
            }
        } else {
            nav.is_err = true;
            continue;
        }
    }
}
