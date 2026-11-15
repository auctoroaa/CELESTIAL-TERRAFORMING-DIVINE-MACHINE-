//! Stable CPU-to-GPU camera uniform layout. Matrices are column-major.
//! The consuming shader must use this exact field order and metadata contract.

use bytemuck::{Pod, Zeroable};
use glam::{DVec3, Mat4};
use crate::packet::{split_dvec3, PacketError};

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GpuCameraPacket {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub view_projection: [[f32; 4]; 4],
    pub inverse_view_projection: [[f32; 4]; 4],
    pub origin_high: [f32; 4],
    pub origin_low: [f32; 4],
    /// width, height, reciprocal width, reciprocal height
    pub viewport: [f32; 4],
    /// near, projection far (0 for infinite), visibility distance, atmospheric horizon
    pub clip_parameters: [f32; 4],
    /// x = ClipSpace discriminant, y = DepthDirection discriminant,
    /// z = flags bitfield, w = ABI version (currently 1).
    pub metadata: [u32; 4],
}

impl GpuCameraPacket {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        view: Mat4,
        projection: Mat4,
        view_projection: Mat4,
        inverse_view_projection: Mat4,
        origin: DVec3,
        viewport: [f32; 2],
        clip_parameters: [f32; 4],
        clip_space: u32,
        depth_direction: u32,
        flags: u32,
    ) -> Result<Self, PacketError> {
        let matrices = [view, projection, view_projection, inverse_view_projection];
        if matrices.iter().any(|m| !m.is_finite())
            || viewport.iter().any(|v| !v.is_finite() || *v <= 0.0)
            || clip_parameters.iter().any(|v| !v.is_finite())
        {
            return Err(PacketError::NonFinite);
        }
        if clip_parameters[0] <= 0.0
            || (clip_parameters[1] != 0.0 && clip_parameters[1] <= clip_parameters[0])
            || clip_parameters[2] < 0.0
            || clip_parameters[3] < 0.0
        {
            return Err(PacketError::InvalidClip);
        }
        let split = split_dvec3(origin)?;
        Ok(Self {
            view: view.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
            view_projection: view_projection.to_cols_array_2d(),
            inverse_view_projection: inverse_view_projection.to_cols_array_2d(),
            origin_high: [split.high.x, split.high.y, split.high.z, 0.0],
            origin_low: [split.low.x, split.low.y, split.low.z, 0.0],
            viewport: [viewport[0], viewport[1], 1.0 / viewport[0], 1.0 / viewport[1]],
            clip_parameters,
            metadata: [clip_space, depth_direction, flags, 1],
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

const _: [(); 336] = [(); core::mem::size_of::<GpuCameraPacket>()];
const _: [(); 16] = [(); core::mem::align_of::<GpuCameraPacket>()];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_layout_is_stable() {
        assert_eq!(core::mem::size_of::<GpuCameraPacket>(), 336);
        assert_eq!(core::mem::align_of::<GpuCameraPacket>(), 16);
        assert_eq!(core::mem::offset_of!(GpuCameraPacket, origin_high), 256);
        assert_eq!(core::mem::offset_of!(GpuCameraPacket, clip_parameters), 304);
        assert_eq!(core::mem::offset_of!(GpuCameraPacket, metadata), 320);
    }
}
