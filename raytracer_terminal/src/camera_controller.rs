use std::{collections::HashSet, time::Duration};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use raytracer::{Camera, Scalar, Vec3, get_camera_rotation};

#[derive(Debug, Clone, Default)]
pub struct CameraController {
	last_keys_down: HashSet<KeyCode>,
}
impl CameraController {
	pub fn update(
		&self,
		camera: &mut Camera,
		yaw: &mut Scalar,
		pitch: &mut Scalar,
		width: usize,
		height: usize,
		delta_time: Duration,
	) {
		let mut look_up = if self.last_keys_down.contains(&KeyCode::Up) {
			1.0
		} else {
			0.0
		};
		if self.last_keys_down.contains(&KeyCode::Down) {
			look_up -= 1.0;
		}
		let mut look_left = if self.last_keys_down.contains(&KeyCode::Left) {
			1.0
		} else {
			0.0
		};
		if self.last_keys_down.contains(&KeyCode::Right) {
			look_left -= 1.0;
		}

		const LOOK_SENSITIVITY: Scalar = 0.5;
		*yaw -= look_left * LOOK_SENSITIVITY;
		*pitch += look_up * LOOK_SENSITIVITY;
		*pitch = (*pitch).clamp(-90.0, 90.0);
		let direction = get_camera_rotation(*yaw, *pitch);

		const MOVE_SPEED: Scalar = 5.0;

		let mut move_forward = if self.last_keys_down.contains(&KeyCode::Char('w')) {
			1.0
		} else {
			0.0
		};
		if self.last_keys_down.contains(&KeyCode::Char('s')) {
			move_forward -= 1.0;
		}
		let mut move_left = if self.last_keys_down.contains(&KeyCode::Char('d')) {
			1.0
		} else {
			0.0
		};
		if self.last_keys_down.contains(&KeyCode::Char('a')) {
			move_left -= 1.0;
		}

		let mut move_dir = direction * move_forward + direction.cross(Camera::WORLD_UP) * move_left;
		move_dir = if move_dir == Vec3::zero() {
			Vec3::zero()
		} else {
			move_dir.normalize()
		};
		let move_multiplier = delta_time.as_secs_f64() * MOVE_SPEED;

		camera.origin = camera.origin + move_dir * move_multiplier;
		*camera = Camera::new(
			camera.origin,
			direction,
			Camera::DEFAULT_FOV,
			Camera::DEFAULT_FOCUS_DIST,
			Camera::DEFAULT_DEFOCUS_ANGLE,
			width,
			height,
		);
	}

	pub fn on_key_event(&mut self, key_event: KeyEvent) {
		match key_event.kind {
			KeyEventKind::Press | KeyEventKind::Repeat => {
				self.last_keys_down.insert(key_event.code);
			}
			KeyEventKind::Release => {
				self.last_keys_down.remove(&key_event.code);
			}
		}
	}
}
