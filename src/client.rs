use ambient_api::prelude::*;
use packages::this::messages::Paint;

#[main]
pub fn main() {
    fixed_rate_tick(Duration::from_millis(20), move |_| {
        let Some(camera_id) = camera::get_active() else {
            return;
        };

        let input = input::get();
        if input.keys.contains(&KeyCode::Q) {
            let ray = camera::clip_position_to_world_ray(camera_id, Vec2::ZERO);

            Paint {
                ray_origin: ray.origin,
                ray_dir: ray.dir,
            }
            .send_server_unreliable();
        }
    });
}
