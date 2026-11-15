//! Deterministic, renderer-independent planetary camera contracts.
//! Camera observations are read-only: this crate never mutates planetary state.

pub mod gpu_abi;
pub mod packet;
pub mod replay;

use glam::{DQuat, DVec3, Vec3};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CameraError {
    #[error("camera value is non-finite or outside its valid range")]
    InvalidValue,
    #[error("unsupported or incompatible camera contract")]
    ContractMismatch,
    #[error(transparent)]
    Packet(#[from] packet::PacketError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraMode {
    FirstPerson,
    ThirdPerson,
    OverheadSculpt,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraState {
    pub mode: CameraMode,
    /// Authoritative planetary coordinate. Never narrow this to f32 before subtraction.
    pub world_position: DVec3,
    pub world_rotation: DQuat,
    pub local_up: DVec3,
}

impl CameraState {
    pub fn validate(&self) -> Result<(), CameraError> {
        let rotation_length = self.world_rotation.length_squared();
        let up_length = self.local_up.length_squared();
        const UNIT_TOLERANCE: f64 = 1.0e-6;

        if !self.world_position.is_finite()
            || !self.world_rotation.is_finite()
            || !rotation_length.is_finite()
            || (rotation_length - 1.0).abs() > UNIT_TOLERANCE
            || !self.local_up.is_finite()
            || !up_length.is_finite()
            || (up_length - 1.0).abs() > UNIT_TOLERANCE
        {
            return Err(CameraError::InvalidValue);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl Viewport {
    pub fn validate(self) -> Result<(), CameraError> {
        if self.width == 0 || self.height == 0 {
            return Err(CameraError::InvalidValue);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraObservation {
    pub state: CameraState,
    pub viewport: Viewport,
    pub view_projection: [[f32; 4]; 4],
    pub near_plane: f64,
    /// None means the projection itself has no finite far plane.
    pub projection_far: Option<f64>,
}

impl CameraObservation {
    pub fn validate(&self) -> Result<(), CameraError> {
        self.state.validate()?;
        self.viewport.validate()?;
        if !self.near_plane.is_finite()
            || self.near_plane <= 0.0
            || self
                .projection_far
                .is_some_and(|v| !v.is_finite() || v <= self.near_plane)
            || self
                .view_projection
                .iter()
                .flatten()
                .any(|v| !v.is_finite())
        {
            return Err(CameraError::InvalidValue);
        }
        Ok(())
    }

    /// Convert a planetary point to camera-relative f32 only after f64 subtraction.
    pub fn relative_position(&self, world: DVec3) -> Result<Vec3, CameraError> {
        Ok(packet::camera_relative_f32(
            world,
            self.state.world_position,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_state() -> CameraState {
        CameraState {
            mode: CameraMode::FirstPerson,
            world_position: DVec3::ZERO,
            world_rotation: DQuat::IDENTITY,
            local_up: DVec3::Y,
        }
    }

    #[test]
    fn accepts_unit_camera_orientation_and_up() {
        assert!(valid_state().validate().is_ok());
    }

    #[test]
    fn rejects_non_unit_camera_orientation() {
        let mut state = valid_state();
        state.world_rotation = DQuat::from_xyzw(0.0, 0.0, 0.0, 2.0);
        assert_eq!(state.validate(), Err(CameraError::InvalidValue));
    }

    #[test]
    fn rejects_non_unit_local_up() {
        let mut state = valid_state();
        state.local_up = DVec3::new(0.0, 2.0, 0.0);
        assert_eq!(state.validate(), Err(CameraError::InvalidValue));
    }
}
