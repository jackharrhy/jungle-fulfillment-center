use ambient_api::{
    core::{
        app::components::name,
        ecs::components::remove_at_game_time,
        hierarchy::components::children,
        model::components::model_from_url,
        physics::components::{cube_collider, dynamic, plane_collider, sphere_collider},
        player::components::is_player,
        prefab::components::prefab_from_url,
        primitives::{
            components::{cube, quad},
            concepts::Sphere,
        },
        rendering::components::color,
        transform::{
            components::{rotation, scale, translation},
            concepts::{Transformable, TransformableOptional},
        },
    },
    physics::add_force,
    prelude::*,
};
use packages::{
    character_animation::components::basic_character_animations,
    character_controller::components::use_character_controller,
    this::{
        components::{held_by, holdable},
        messages::Interact,
        types::InteractState,
    },
};

fn build_floor() {
    Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.0)
        .with(color(), vec4(1.0, 0.0, 0.0, 1.0))
        .with(plane_collider(), ())
        .spawn();
}

fn rain_spheres() {
    fixed_rate_tick(Duration::from_secs_f32(0.5), |_| {
        Entity::new()
            .with_merge(Sphere::suggested())
            .with_merge(Transformable::suggested())
            .with(scale(), Vec3::ONE * 0.2)
            .with(
                translation(),
                Vec3::X * 10. + (random::<Vec2>() * 2.0 - 1.0).extend(10.),
            )
            .with(sphere_collider(), 0.5)
            .with(dynamic(), true)
            .with(remove_at_game_time(), game_time() + Duration::from_secs(5))
            .spawn();
    });
}

fn build_shute() {
    Entity::new()
        .with_merge(Transformable {
            local_to_world: Default::default(),
            optional: TransformableOptional {
                scale: Some(Vec3::ONE * 1.),
                translation: Some(vec3(10., 0., 3.)),
                ..Default::default()
            },
        })
        .with(prefab_from_url(), packages::this::assets::url("shute.glb"))
        .spawn();
}

fn build_random_cubes() {
    for _ in 0..30 {
        Entity::new()
            .with(cube(), ())
            .with(cube_collider(), Vec3::ONE)
            .with(dynamic(), true)
            .with(holdable(), ())
            .with(translation(), (random::<Vec2>() * 20.0 - 10.0).extend(1.))
            .spawn();
    }
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

            let None = entity::get_component(hit.entity, color()) else {
                return;
            };

            entity::add_components(
                hit.entity,
                Entity::new()
                    .with(color(), vec4(0., 1., 0., 1.))
                    .with(held_by(), client_entity_id),
            )
        } else {
            let held_entities = held_by_query.evaluate();

            let Some((held_entity, _)) = held_entities
                .iter()
                .find(|(_, holder)| *holder == client_entity_id)
            else {
                return;
            };

            entity::remove_component(*held_entity, held_by());
            entity::remove_component(*held_entity, color());
        }
    });
}

fn apply_force_to_held_entities() {
    let held_by_query = query(held_by()).build();

    let cube = Entity::new()
        .with(cube(), ())
        .with(translation(), Vec3::new(0., 0., 3.))
        .with(scale(), Vec3::ONE * 0.5)
        .with(rotation(), Quat::IDENTITY)
        .spawn();

    fixed_rate_tick(Duration::from_millis(5), move |_| {
        let held_entities = held_by_query.evaluate();

        for (held, player) in held_entities {
            add_force(held, vec3(0., 0., 100.));

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

            let forward = vec3(0., 0., 2.);
            let looking = player_rot * head_rot;
            let held_trans = (player_trans + (head_trans * 0.65)) + looking.mul_vec3(forward);

            entity::set_component(cube, translation(), held_trans);
            entity::set_component(cube, rotation(), looking);
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
                    .with(basic_character_animations(), id),
            );
        }
    });
}

#[main]
pub fn main() {
    build_floor();

    build_shute();
    rain_spheres();

    build_random_cubes();

    listen_for_interact();
    apply_force_to_held_entities();

    listen_for_players();
}
