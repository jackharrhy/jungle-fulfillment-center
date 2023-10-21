use ambient_api::{core::messages::Frame, input::is_game_focused, prelude::*};
use packages::this::{messages::Interact, types::InteractState};

fn send_interaction(interaction: InteractState) {
    let Some(camera_id) = camera::get_active() else {
        return;
    };

    let ray = camera::clip_position_to_world_ray(camera_id, Vec2::ZERO);

    Interact {
        ray_origin: ray.origin,
        ray_dir: ray.dir,
        interaction: interaction,
    }
    .send_server_unreliable();
}

#[main]
pub fn main() {
    Frame::subscribe(move |_| {
        if !is_game_focused() {
            return;
        }
        let (delta, _input) = input::get_delta();

        if !delta.mouse_buttons.is_empty() && delta.mouse_buttons.contains(&MouseButton::Left) {
            send_interaction(InteractState::Pickup);
        }
        if !delta.mouse_buttons_released.is_empty()
            && delta.mouse_buttons_released.contains(&MouseButton::Left)
        {
            send_interaction(InteractState::LetGo);
        }
    });
}
