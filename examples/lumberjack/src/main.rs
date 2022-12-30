use bevy::prelude::*;
use bevy_goap::{
    Action, ActionState, Actor, ActorState, Condition, EvaluationResult, GoapPlugin, GoapStage,
};
use environment::*;
use navigation::{navigation_system, Navigation};

mod environment;
mod navigation;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugin(GoapPlugin)
        .add_startup_system(setup)
        .add_startup_system(create_lumberjack)
        .add_startup_system(create_axes_system)
        .add_startup_system(create_trees_system)
        .add_system_set_to_stage(
            GoapStage::Actions,
            SystemSet::new()
                .with_system(get_axe_action_system)
                .with_system(chop_tree_action_system)
                .with_system(collect_wood_action_system),
        )
        .add_system_set_to_stage(
            GoapStage::Actors,
            SystemSet::new().with_system(lumberjack_actor_system),
        )
        .add_system(navigation_system);

    #[cfg(feature = "inspector")]
    app.add_plugin(bevy_goap::inspector::GoapInspectorPlugin);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 1000., 1000.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 1024. })),
        material: materials.add(Color::GREEN.into()),
        ..Default::default()
    });
}

fn create_lumberjack(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let get_axe_action = Action::build(GetAxeAction::default())
        .with_precondition(ActorHasAxeCondition, false)
        .with_postcondition(ActorHasAxeCondition, true);

    let chop_tree_action = Action::build(ChopTreeAction::new(5))
        .with_precondition(ActorHasAxeCondition, true)
        .with_postcondition(ActorHasWoodCondition, true);

    let _collect_wood_action = Action::build(CollectWoodAction)
        .with_precondition(ActorHasWoodCondition, false)
        .with_postcondition(ActorHasWoodCondition, true);

    // Try toggling the initial conditions and removing actions to observe the different action sequences the lumberjack performs!
    let lumberjack = Actor::build(Lumberjack)
        .with_initial_condition(ActorHasAxeCondition, false)
        .with_initial_condition(ActorHasWoodCondition, false)
        .with_goal(ActorHasWoodCondition, true)
        .with_action(get_axe_action)
        // .with_action(_collect_wood_action) // Try uncommenting this action to observe a different action sequence the lumberjack performs!
        .with_action(chop_tree_action);

    commands.spawn_empty().insert(lumberjack).insert(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 32. })),
        material: materials.add(Color::RED.into()),
        ..Default::default()
    });
}

#[derive(Component, Clone)]
struct Lumberjack;

#[allow(clippy::type_complexity)]
fn lumberjack_actor_system(
    mut query: Query<(&mut ActorState, &mut Actor), (With<Lumberjack>, Changed<ActorState>)>,
) {
    for (mut actor_state, mut actor) in query.iter_mut() {
        println!("Found changed actor_state to {:?}", actor_state);

        if let ActorState::CompletedPlan = *actor_state {
            actor.update_current_state(ActorHasWoodCondition, false);
            *actor_state = ActorState::RequiresPlan;
        };
    }
}

#[derive(Default, Component, Clone)]
struct GetAxeAction {
    axe_entity: Option<Entity>,
}

fn get_axe_action_system(
    mut commands: Commands,
    mut query: Query<(&mut GetAxeAction, &Action, &mut ActionState), With<GetAxeAction>>,
    actor_transforms_query: Query<&Transform, With<Lumberjack>>,
    axes: Query<(Entity, &Transform), With<Axe>>,
    navigations: Query<&Navigation>,
) {
    for (mut get_axe_action, action, mut action_state) in query.iter_mut() {
        match *action_state {
            ActionState::Evaluate => {
                println!("Evaluating GetAxeAction");

                // Is there an axe available?
                if axes.iter().count() > 0 {
                    *action_state = ActionState::EvaluationComplete(EvaluationResult::Success);
                } else {
                    *action_state = ActionState::EvaluationComplete(EvaluationResult::Failure);
                }
            }
            ActionState::NotInPlan(_) => {
                *action_state = ActionState::Idle;
            }
            ActionState::Started => {
                println!("Starting GetAxeAction");

                let actor_pos = actor_transforms_query
                    .get(action.actor_entity)
                    .unwrap()
                    .translation;

                let closest_axe = axes.iter().min_by_key(|(_, axe_transform)| {
                    (axe_transform.translation - actor_pos).length() as i32
                });

                if let Some((axe_entity, _)) = closest_axe {
                    get_axe_action.axe_entity = Some(axe_entity);

                    commands.entity(action.actor_entity).insert(Navigation {
                        navigator: action.actor_entity,
                        target: axe_entity,
                        speed: 50.,
                        is_done: false,
                    });

                    *action_state = ActionState::Executing;
                } else {
                    *action_state = ActionState::Failure;
                }
            }
            ActionState::Executing => {
                println!("Getting axe!");

                if navigations.get(action.actor_entity).unwrap().is_done {
                    *action_state = ActionState::Complete;
                }
            }
            _ => {}
        };
    }
}

struct ActorHasAxeCondition;
impl Condition for ActorHasAxeCondition {}

#[derive(Component, Clone)]
struct ChopTreeAction {
    tree_entity: Option<Entity>,
    max_chops: u8,
    current_chops: u8,
}

impl ChopTreeAction {
    fn new(max_chops: u8) -> Self {
        Self {
            tree_entity: None,
            max_chops,
            current_chops: 0,
        }
    }
}

fn chop_tree_action_system(
    mut commands: Commands,
    mut query: Query<(&mut ActionState, &mut ChopTreeAction, &Action)>,
    actor_transforms_query: Query<&Transform, With<Lumberjack>>,
    trees: Query<(Entity, &Transform), With<Tree>>,
    navigations: Query<&Navigation>,
) {
    for (mut action_state, mut chop_tree_action, action) in query.iter_mut() {
        match *action_state {
            ActionState::Evaluate => {
                println!("Evaluating ChopTreeAction");

                // Is there an axe available?
                if trees.iter().count() > 0 {
                    *action_state = ActionState::EvaluationComplete(EvaluationResult::Success);
                } else {
                    *action_state = ActionState::EvaluationComplete(EvaluationResult::Failure);
                }
            }
            ActionState::NotInPlan(_) => {
                *action_state = ActionState::Idle;
            }
            ActionState::Started => {
                println!("Starting ChopTreeAction");

                let actor_pos = actor_transforms_query
                    .get(action.actor_entity)
                    .unwrap()
                    .translation;

                let closest_tree = trees.iter().min_by_key(|(_, tree_transform)| {
                    (tree_transform.translation - actor_pos).length() as i32
                });

                if let Some((tree_entity, _)) = closest_tree {
                    chop_tree_action.tree_entity = Some(tree_entity);

                    commands.entity(action.actor_entity).insert(Navigation {
                        navigator: action.actor_entity,
                        target: tree_entity,
                        speed: 50.,
                        is_done: false,
                    });

                    *action_state = ActionState::Executing;
                } else {
                    *action_state = ActionState::Failure;
                }
            }
            ActionState::Executing => {
                if navigations.get(action.actor_entity).unwrap().is_done {
                    chop_tree_action.current_chops += 1;
                    println!("Chopped tree {} times!", chop_tree_action.current_chops);

                    if chop_tree_action.current_chops >= chop_tree_action.max_chops {
                        commands
                            .entity(chop_tree_action.tree_entity.unwrap())
                            .despawn_recursive();

                        chop_tree_action.current_chops = 0;

                        *action_state = ActionState::Complete;
                    }
                } else {
                    println!("Moving to tree!");
                }
            }
            _ => {}
        }
    }
}

#[derive(Component, Clone)]
struct CollectWoodAction;

fn collect_wood_action_system(mut query: Query<&mut ActionState, With<CollectWoodAction>>) {
    for mut action_state in query.iter_mut() {
        match *action_state {
            ActionState::Evaluate => {
                *action_state = ActionState::EvaluationComplete(EvaluationResult::Success);
            }
            ActionState::NotInPlan(_) => {
                *action_state = ActionState::Idle;
            }
            ActionState::Started => {
                println!("Starting to collect wood!");
                *action_state = ActionState::Executing;
            }
            ActionState::Executing => {
                println!("Collecting wood!");
                *action_state = ActionState::Complete;
            }
            _ => (),
        }
    }
}

struct ActorHasWoodCondition;
impl Condition for ActorHasWoodCondition {}
