use std::f32::consts::PI;

use ambient_api::{
    core::{
        app::components::{main_scene, name},
        hierarchy::components::children,
        messages::Frame,
        model::components::model_from_url,
        physics::components::{cube_collider, dynamic},
        player::components::is_player,
        prefab::components::prefab_from_url,
        primitives::components::cube,
        rendering::components::{cast_shadows, fog_density, light_diffuse, sky, sun},
        transform::{
            components::{rotation, translation},
            concepts::{Transformable, TransformableOptional},
        },
    },
    element::{use_frame, use_state},
    physics::add_force,
    prelude::*,
};
use packages::{
    character_animation::components::basic_character_animations,
    character_controller::components::{camera_distance, use_character_controller},
    this::{
        components::{held_by, holdable, score},
        messages::Interact,
        types::InteractState,
    },
};

const GRAVITY: f32 = 9.82;

fn make_sky_and_sun() {
    Entity::new().with(sky(), ()).spawn();

    let sun = Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Quat::IDENTITY)
        .with(main_scene(), ())
        .with(light_diffuse(), vec3(1.0, 1.0, 1.0))
        .with(fog_density(), 0.)
        .spawn();

    Frame::subscribe(move |_| {
        let time = game_time().as_secs_f32();
        let rot = Quat::from_axis_angle(vec3(0.0, 1.0, 0.4).normalize(), (time * 0.1) + PI);
        entity::set_component(sun, rotation(), rot);
    });
}

fn build_level() {
    Entity::new()
        .with_merge(Transformable {
            local_to_world: Default::default(),
            optional: TransformableOptional {
                scale: Some(Vec3::ONE * 1.),
                ..Default::default()
            },
        })
        .with(prefab_from_url(), packages::this::assets::url("level.glb"))
        .with(cast_shadows(), ())
        .spawn();
}

fn make_box() -> Entity {
    let starting_point = vec3(0., 10., 10.);
    let spread = 5.;

    Entity::new()
        .with(cube(), ())
        .with(translation(), starting_point)
        .spawn();

    Entity::new()
        .with(cube(), ())
        .with(cube_collider(), Vec3::ONE)
        .with(dynamic(), true)
        .with(holdable(), ())
        .with(
            translation(),
            starting_point + (random::<Vec2>() * spread - (spread / 2.)).extend(1.),
        )
}

fn rain_boxes() {
    fixed_rate_tick(Duration::from_secs_f32(2.0), |_| {
        make_box().spawn();
    });
}

fn listen_for_interact() {
    let held_by_query = query(held_by()).build();

    Interact::subscribe(move |ctx, msg| {
        if ctx.client_user_id().is_none() {
            return;
        }

        let Some(client_entity_id) = ctx.client_entity_id() else {
            return;
        };

        if msg.interaction == InteractState::Pickup {
            let Some(hit) = physics::raycast_first(msg.ray_origin, msg.ray_dir) else {
                return;
            };

            if !entity::has_component(hit.entity, holdable()) {
                return;
            }

            entity::add_components(hit.entity, Entity::new().with(held_by(), client_entity_id))
        } else {
            let held_entities = held_by_query.evaluate();

            let Some((held_entity, _)) = held_entities
                .iter()
                .find(|(_, holder)| *holder == client_entity_id)
            else {
                return;
            };

            entity::remove_component(*held_entity, held_by());
        }
    });
}

fn apply_force_to_held_entities() {
    let held_by_query = query(held_by()).build();

    fixed_rate_tick(Duration::from_millis(5), move |_| {
        let held_entities = held_by_query.evaluate();

        for (held, player) in held_entities {
            let Some(children) = entity::get_component(player, children()) else {
                return;
            };

            let Some(head) = children.iter().find(|entity| {
                let Some(name) = entity::get_component(**entity, name()) else {
                    return false;
                };
                return name == "Head";
            }) else {
                return;
            };

            let Some(player_trans) = entity::get_component(player, translation()) else {
                return;
            };
            let Some(player_rot) = entity::get_component(player, rotation()) else {
                return;
            };
            let Some(head_trans) = entity::get_component(*head, translation()) else {
                return;
            };
            let Some(head_rot) = entity::get_component(*head, rotation()) else {
                return;
            };

            let forward = vec3(0., 0., 4.);
            let looking = player_rot * head_rot;
            let held_dest_trans = (player_trans + (head_trans * 0.65)) + looking.mul_vec3(forward);

            let Some(held_trans) = entity::get_component(held, translation()) else {
                return;
            };

            let force_vec = (held_dest_trans - held_trans).normalize();

            let max_force = 300.;
            let distance = held_dest_trans.distance(held_trans);

            let wanted_force = distance * (max_force / 4.);
            let force = if wanted_force > max_force {
                max_force
            } else {
                wanted_force
            };

            add_force(held, vec3(0., 0., GRAVITY));
            add_force(held, force_vec * force);
        }
    });
}

fn listen_for_players() {
    spawn_query(is_player()).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with(use_character_controller(), ())
                    .with(
                        model_from_url(),
                        packages::base_assets::assets::url("Y Bot.fbx"),
                    )
                    .with(basic_character_animations(), id)
                    .with(camera_distance(), -1.),
            );
        }
    });
}

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let (holdable_count, set_holdable_count) = use_state(hooks, 0);
    let (score_number, set_score_number) = use_state(hooks, 0);

    let holdable_by_query = query(holdable()).build();
    let score_query = query(score()).build();

    use_frame(hooks, move |_world| {
        let holdable = holdable_by_query.evaluate();

        set_holdable_count(holdable.len());

        let scores = score_query.evaluate();
        for (score_entity, _) in scores {
            let Some(score) = entity::get_component(score_entity, score()) else {
                continue;
            };

            set_score_number(score);
        }
    });

    FlowColumn::el([Text::el(format!(
        "You have a score of {score_number}, there are {holdable_count} boxes left to pick up."
    ))])
    .with_padding_even(STREET)
    .with(space_between_items(), 100.)
}

fn despawn_and_count_disposed_cubes() {
    let holdable_by_query = query(holdable()).build();

    let score_entity = Entity::new().with(score(), 0).spawn();

    Frame::subscribe(move |_| {
        let holdables = holdable_by_query.evaluate();

        for (holdable, _) in holdables {
            let Some(holdable_trans) = entity::get_component(holdable, translation()) else {
                return;
            };

            if holdable_trans.z < -4. {
                entity::despawn(holdable);
                entity::mutate_component(score_entity, score(), |old_score| *old_score += 1);
            }
        }
    });
}

#[main]
pub fn main() {
    make_sky_and_sun();
    build_level();

    rain_boxes();

    listen_for_interact();
    apply_force_to_held_entities();
    despawn_and_count_disposed_cubes();

    listen_for_players();

    App.el().spawn_interactive();
}
