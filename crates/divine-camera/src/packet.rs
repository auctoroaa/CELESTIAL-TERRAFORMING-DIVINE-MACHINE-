//! Precision-safe render packet primitives and explicit depth conventions.

use glam::{DVec3, Vec3, Vec4};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipSpace {
    ZeroToOne,
    NegativeOneToOne,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepthDirection {
    Forward,
    Reversed,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SplitOrigin {
    pub high: Vec3,
    pub low: Vec3,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PacketError {
    #[error("coordinate or matrix contains a non-finite value")]
    NonFinite,
    #[error("coordinate cannot be represented by the GPU packet")]
    OutOfRange,
    #[error("invalid clipping configuration")]
    InvalidClip,
}

pub fn split_dvec3(value: DVec3) -> Result<SplitOrigin, PacketError> {
    if !value.is_finite() {
        return Err(PacketError::NonFinite);
    }
    let high = value.as_vec3();
    if !high.is_finite() {
        return Err(PacketError::OutOfRange);
    }
    let low64 = value - high.as_dvec3();
    let low = low64.as_vec3();
    if !low.is_finite() {
        return Err(PacketError::OutOfRange);
    }
    Ok(SplitOrigin { high, low })
}

pub fn camera_relative_f32(point: DVec3, camera: DVec3) -> Result<Vec3, PacketError> {
    if !point.is_finite() || !camera.is_finite() {
        return Err(PacketError::NonFinite);
    }
    let relative = (point - camera).as_vec3();
    if !relative.is_finite() {
        return Err(PacketError::OutOfRange);
    }
    Ok(relative)
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Plane {
    Active(Vec4),
    Disabled,
}

pub fn normalize_plane(plane: Vec4) -> Result<Plane, PacketError> {
    if !plane.is_finite() {
        return Err(PacketError::NonFinite);
    }
    let length = plane.truncate().length();
    if !length.is_finite() {
        return Err(PacketError::NonFinite);
    }
    if length <= 1.0e-7 {
        return Ok(Plane::Disabled);
    }
    Ok(Plane::Active(plane / length))
}

pub fn validate_matrix(matrix: &[[f32; 4]; 4]) -> Result<(), PacketError> {
    if matrix.iter().flatten().any(|v| !v.is_finite()) {
        return Err(PacketError::NonFinite);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_origin_reconstructs_large_coordinates() {
        let p = DVec3::new(6.4e9 + 0.125, -2.1e9 + 0.25, 9.0e8 + 0.5);
        let s = split_dvec3(p).unwrap();
        let reconstructed = s.high.as_dvec3() + s.low.as_dvec3();
        assert!((p - reconstructed).abs().max_element() < 1.0e-5);
    }

    #[test]
    fn subtract_before_narrowing_preserves_local_detail() {
        let origin = DVec3::splat(6.0e9);
        let point = origin + DVec3::new(0.25, 0.5, 0.75);
        assert_eq!(
            camera_relative_f32(point, origin).unwrap(),
            Vec3::new(0.25, 0.5, 0.75)
        );
    }

    #[test]
    fn rejects_unrepresentable_gpu_coordinates() {
        assert_eq!(
            split_dvec3(DVec3::splat(f64::MAX)),
            Err(PacketError::OutOfRange)
        );
    }
}
