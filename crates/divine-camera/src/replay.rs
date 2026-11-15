//! Sealed deterministic camera replay with one ordered world-query stream.
//! Playback fails closed: query mismatch, exhaustion, or unconsumed events are errors.

use glam::{DVec3, Vec3};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurfaceSample {
    pub position: DVec3,
    pub normal: Vec3,
    pub surface_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldRay {
    pub origin: DVec3,
    pub direction: Vec3,
    pub min_distance: f64,
    pub max_distance: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerrainHit {
    pub position: DVec3,
    pub normal: Vec3,
    pub distance: f64,
    pub surface_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorldQuery {
    SampleSurface { position: DVec3, result: Option<SurfaceSample> },
    ResolveSegment { origin: DVec3, desired: DVec3, radius: f64, result: DVec3 },
    RaycastTerrain { ray: WorldRay, result: Option<TerrainHit> },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorldQueryError {
    #[error("recorded world-query stream exhausted")]
    Exhausted,
    #[error("world-query kind or input diverged at event {index}")]
    Diverged { index: usize },
    #[error("recorded world-query stream contains unconsumed events")]
    Unconsumed,
    #[error("invalid world-query input")]
    InvalidInput,
}

pub trait PlanetaryWorld {
    fn sample_surface(&mut self, position: DVec3) -> Result<Option<SurfaceSample>, WorldQueryError>;
    fn resolve_camera_segment(&mut self, origin: DVec3, desired: DVec3, radius: f64) -> Result<DVec3, WorldQueryError>;
    fn raycast_terrain(&mut self, ray: &WorldRay) -> Result<Option<TerrainHit>, WorldQueryError>;
    fn finish_step(&mut self) -> Result<(), WorldQueryError>;
}

#[derive(Debug, Clone)]
pub struct RecordedWorldPlayback {
    events: Vec<WorldQuery>,
    cursor: usize,
}

impl RecordedWorldPlayback {
    pub fn new(events: Vec<WorldQuery>) -> Self { Self { events, cursor: 0 } }

    fn take(&mut self) -> Result<WorldQuery, WorldQueryError> {
        let index = self.cursor;
        let event = self.events.get(index).cloned().ok_or(WorldQueryError::Exhausted)?;
        self.cursor += 1;
        Ok(event)
    }

    pub fn consumed(&self) -> usize { self.cursor }
}

impl PlanetaryWorld for RecordedWorldPlayback {
    fn sample_surface(&mut self, position: DVec3) -> Result<Option<SurfaceSample>, WorldQueryError> {
        match self.take()? {
            WorldQuery::SampleSurface { position: expected, result } if expected == position => Ok(result),
            _ => Err(WorldQueryError::Diverged { index: self.cursor - 1 }),
        }
    }

    fn resolve_camera_segment(&mut self, origin: DVec3, desired: DVec3, radius: f64) -> Result<DVec3, WorldQueryError> {
        if !radius.is_finite() || radius < 0.0 { return Err(WorldQueryError::InvalidInput); }
        match self.take()? {
            WorldQuery::ResolveSegment { origin: eo, desired: ed, radius: er, result }
                if eo == origin && ed == desired && er.to_bits() == radius.to_bits() => Ok(result),
            _ => Err(WorldQueryError::Diverged { index: self.cursor - 1 }),
        }
    }

    fn raycast_terrain(&mut self, ray: &WorldRay) -> Result<Option<TerrainHit>, WorldQueryError> {
        validate_ray(ray)?;
        match self.take()? {
            WorldQuery::RaycastTerrain { ray: expected, result } if expected == *ray => Ok(result),
            _ => Err(WorldQueryError::Diverged { index: self.cursor - 1 }),
        }
    }

    fn finish_step(&mut self) -> Result<(), WorldQueryError> {
        if self.cursor == self.events.len() { Ok(()) } else { Err(WorldQueryError::Unconsumed) }
    }
}

fn validate_ray(ray: &WorldRay) -> Result<(), WorldQueryError> {
    if !ray.origin.is_finite() || !ray.direction.is_finite()
        || (ray.direction.length_squared() - 1.0).abs() > 1.0e-4
        || !ray.min_distance.is_finite() || !ray.max_distance.is_finite()
        || ray.min_distance < 0.0 || ray.max_distance <= ray.min_distance
    { return Err(WorldQueryError::InvalidInput); }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayPayload<T> {
    pub format_version: u32,
    pub contract_version: u32,
    pub payload: T,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplaySeal {
    pub algorithm: String,
    pub digest_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReplayIntegrityError {
    #[error("canonical serialization failed")]
    Serialization,
    #[error("replay seal mismatch")]
    SealMismatch,
    #[error("unsupported replay format or camera contract")]
    VersionMismatch,
}

pub fn seal_replay<T: Serialize>(payload: &ReplayPayload<T>) -> Result<ReplaySeal, ReplayIntegrityError> {
    let bytes = serde_json::to_vec(payload).map_err(|_| ReplayIntegrityError::Serialization)?;
    Ok(ReplaySeal { algorithm: "BLAKE3".into(), digest_hex: blake3::hash(&bytes).to_hex().to_string() })
}

pub fn verify_replay<T: Serialize>(payload: &ReplayPayload<T>, seal: &ReplaySeal) -> Result<(), ReplayIntegrityError> {
    if payload.format_version != 1 || payload.contract_version != 2 {
        return Err(ReplayIntegrityError::VersionMismatch);
    }
    let actual = seal_replay(payload)?;
    if seal.algorithm != actual.algorithm || seal.digest_hex != actual.digest_hex {
        return Err(ReplayIntegrityError::SealMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_uses_single_ordered_stream_and_checks_consumption() {
        let p = DVec3::new(1.0, 2.0, 3.0);
        let events = vec![
            WorldQuery::SampleSurface { position: p, result: None },
            WorldQuery::ResolveSegment { origin: p, desired: p + DVec3::X, radius: 0.5, result: p },
        ];
        let mut world = RecordedWorldPlayback::new(events);
        assert_eq!(world.sample_surface(p).unwrap(), None);
        assert_eq!(world.resolve_camera_segment(p, p + DVec3::X, 0.5).unwrap(), p);
        assert_eq!(world.finish_step(), Ok(()));
    }

    #[test]
    fn playback_rejects_reordered_or_wrong_inputs() {
        let p = DVec3::ZERO;
        let mut world = RecordedWorldPlayback::new(vec![
            WorldQuery::SampleSurface { position: p, result: None },
        ]);
        assert!(matches!(world.sample_surface(DVec3::ONE), Err(WorldQueryError::Diverged { index: 0 })));
    }

    #[test]
    fn tampering_invalidates_seal_before_replay() {
        let mut payload = ReplayPayload { format_version: 1, contract_version: 2, payload: vec![1u32, 2] };
        let seal = seal_replay(&payload).unwrap();
        verify_replay(&payload, &seal).unwrap();
        payload.payload[0] = 9;
        assert_eq!(verify_replay(&payload, &seal), Err(ReplayIntegrityError::SealMismatch));
    }
}
