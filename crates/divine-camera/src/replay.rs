//! Sealed deterministic camera replay with one ordered world-query stream.
//! Playback fails closed: invalid data, query mismatch, exhaustion, or unconsumed events are errors.

use glam::{DVec3, Vec3};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

const REPLAY_FORMAT_VERSION: u32 = 1;
const CAMERA_CONTRACT_VERSION: u32 = 2;
const SEAL_ALGORITHM: &str = "BLAKE3-CANONICAL-JSON-V1";

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
    SampleSurface {
        position: DVec3,
        result: Option<SurfaceSample>,
    },
    ResolveSegment {
        origin: DVec3,
        desired: DVec3,
        radius: f64,
        result: DVec3,
    },
    RaycastTerrain {
        ray: WorldRay,
        result: Option<TerrainHit>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorldQueryError {
    #[error("recorded world-query stream exhausted")]
    Exhausted,
    #[error("world-query kind or input diverged at event {index}")]
    Diverged { index: usize },
    #[error("recorded world-query stream contains unconsumed events")]
    Unconsumed,
    #[error("invalid world-query input or recorded result")]
    InvalidInput,
}

pub trait PlanetaryWorld {
    fn sample_surface(&mut self, position: DVec3)
        -> Result<Option<SurfaceSample>, WorldQueryError>;
    fn resolve_camera_segment(
        &mut self,
        origin: DVec3,
        desired: DVec3,
        radius: f64,
    ) -> Result<DVec3, WorldQueryError>;
    fn raycast_terrain(&mut self, ray: &WorldRay) -> Result<Option<TerrainHit>, WorldQueryError>;
    fn finish_step(&mut self) -> Result<(), WorldQueryError>;
}

#[derive(Debug, Clone)]
pub struct RecordedWorldPlayback {
    events: Vec<WorldQuery>,
    cursor: usize,
}

impl RecordedWorldPlayback {
    /// Construct playback without eager validation. Every event is validated before its result is returned.
    pub fn new(events: Vec<WorldQuery>) -> Self {
        Self { events, cursor: 0 }
    }

    /// Validate the complete recorded stream before playback begins.
    pub fn try_new(events: Vec<WorldQuery>) -> Result<Self, WorldQueryError> {
        for event in &events {
            validate_event(event)?;
        }
        Ok(Self::new(events))
    }

    fn take(&mut self) -> Result<WorldQuery, WorldQueryError> {
        let index = self.cursor;
        let event = self
            .events
            .get(index)
            .cloned()
            .ok_or(WorldQueryError::Exhausted)?;
        self.cursor += 1;
        Ok(event)
    }

    pub fn consumed(&self) -> usize {
        self.cursor
    }
}

impl PlanetaryWorld for RecordedWorldPlayback {
    fn sample_surface(
        &mut self,
        position: DVec3,
    ) -> Result<Option<SurfaceSample>, WorldQueryError> {
        if !position.is_finite() {
            return Err(WorldQueryError::InvalidInput);
        }
        match self.take()? {
            WorldQuery::SampleSurface {
                position: expected,
                result,
            } if expected == position => {
                if let Some(sample) = &result {
                    validate_surface_sample(sample)?;
                }
                Ok(result)
            }
            _ => Err(WorldQueryError::Diverged {
                index: self.cursor - 1,
            }),
        }
    }

    fn resolve_camera_segment(
        &mut self,
        origin: DVec3,
        desired: DVec3,
        radius: f64,
    ) -> Result<DVec3, WorldQueryError> {
        if !origin.is_finite() || !desired.is_finite() || !radius.is_finite() || radius < 0.0 {
            return Err(WorldQueryError::InvalidInput);
        }
        match self.take()? {
            WorldQuery::ResolveSegment {
                origin: expected_origin,
                desired: expected_desired,
                radius: expected_radius,
                result,
            } if expected_origin == origin
                && expected_desired == desired
                && expected_radius.to_bits() == radius.to_bits() =>
            {
                if !result.is_finite() {
                    return Err(WorldQueryError::InvalidInput);
                }
                Ok(result)
            }
            _ => Err(WorldQueryError::Diverged {
                index: self.cursor - 1,
            }),
        }
    }

    fn raycast_terrain(&mut self, ray: &WorldRay) -> Result<Option<TerrainHit>, WorldQueryError> {
        validate_ray(ray)?;
        match self.take()? {
            WorldQuery::RaycastTerrain {
                ray: expected,
                result,
            } if expected == *ray => {
                if let Some(hit) = &result {
                    validate_terrain_hit(ray, hit)?;
                }
                Ok(result)
            }
            _ => Err(WorldQueryError::Diverged {
                index: self.cursor - 1,
            }),
        }
    }

    fn finish_step(&mut self) -> Result<(), WorldQueryError> {
        if self.cursor == self.events.len() {
            Ok(())
        } else {
            Err(WorldQueryError::Unconsumed)
        }
    }
}

fn validate_event(event: &WorldQuery) -> Result<(), WorldQueryError> {
    match event {
        WorldQuery::SampleSurface { position, result } => {
            if !position.is_finite() {
                return Err(WorldQueryError::InvalidInput);
            }
            if let Some(sample) = result {
                validate_surface_sample(sample)?;
            }
        }
        WorldQuery::ResolveSegment {
            origin,
            desired,
            radius,
            result,
        } => {
            if !origin.is_finite()
                || !desired.is_finite()
                || !radius.is_finite()
                || *radius < 0.0
                || !result.is_finite()
            {
                return Err(WorldQueryError::InvalidInput);
            }
        }
        WorldQuery::RaycastTerrain { ray, result } => {
            validate_ray(ray)?;
            if let Some(hit) = result {
                validate_terrain_hit(ray, hit)?;
            }
        }
    }
    Ok(())
}

fn validate_unit_normal(normal: Vec3) -> Result<(), WorldQueryError> {
    if !normal.is_finite() || (normal.length_squared() - 1.0).abs() > 1.0e-3 {
        return Err(WorldQueryError::InvalidInput);
    }
    Ok(())
}

fn validate_surface_sample(sample: &SurfaceSample) -> Result<(), WorldQueryError> {
    if !sample.position.is_finite() {
        return Err(WorldQueryError::InvalidInput);
    }
    validate_unit_normal(sample.normal)
}

fn validate_ray(ray: &WorldRay) -> Result<(), WorldQueryError> {
    if !ray.origin.is_finite()
        || !ray.direction.is_finite()
        || (ray.direction.length_squared() - 1.0).abs() > 1.0e-4
        || !ray.min_distance.is_finite()
        || !ray.max_distance.is_finite()
        || ray.min_distance < 0.0
        || ray.max_distance <= ray.min_distance
    {
        return Err(WorldQueryError::InvalidInput);
    }
    Ok(())
}

fn validate_terrain_hit(ray: &WorldRay, hit: &TerrainHit) -> Result<(), WorldQueryError> {
    if !hit.position.is_finite()
        || !hit.distance.is_finite()
        || hit.distance < ray.min_distance
        || hit.distance > ray.max_distance
    {
        return Err(WorldQueryError::InvalidInput);
    }
    validate_unit_normal(hit.normal)?;
    let expected = ray.origin + ray.direction.as_dvec3() * hit.distance;
    let tolerance = 1.0e-6 * hit.distance.abs().max(1.0);
    if (hit.position - expected).abs().max_element() > tolerance {
        return Err(WorldQueryError::InvalidInput);
    }
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

fn write_canonical_json(value: &Value, output: &mut Vec<u8>) -> Result<(), ReplayIntegrityError> {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(value) => output.extend_from_slice(if *value { b"true" } else { b"false" }),
        Value::Number(number) => {
            if number.as_f64().is_some_and(|value| !value.is_finite()) {
                return Err(ReplayIntegrityError::Serialization);
            }
            output.extend_from_slice(number.to_string().as_bytes());
        }
        Value::String(value) => {
            let encoded =
                serde_json::to_vec(value).map_err(|_| ReplayIntegrityError::Serialization)?;
            output.extend_from_slice(&encoded);
        }
        Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_canonical_json(value, output)?;
            }
            output.push(b']');
        }
        Value::Object(values) => {
            output.push(b'{');
            let mut keys: Vec<_> = values.keys().collect();
            keys.sort_unstable();
            for (index, key) in keys.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                let encoded_key =
                    serde_json::to_vec(key).map_err(|_| ReplayIntegrityError::Serialization)?;
                output.extend_from_slice(&encoded_key);
                output.push(b':');
                write_canonical_json(&values[*key], output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

#[derive(Debug)]
struct FiniteValidationError;

impl std::fmt::Display for FiniteValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("non-finite floating-point value")
    }
}

impl std::error::Error for FiniteValidationError {}

impl serde::ser::Error for FiniteValidationError {
    fn custom<T: std::fmt::Display>(_message: T) -> Self {
        Self
    }
}

struct FiniteValidator;

struct ValidatorCompound;

impl serde::ser::SerializeSeq for ValidatorCompound {
    type Ok = ();
    type Error = FiniteValidationError;

    fn serialize_element<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(FiniteValidator)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl serde::ser::SerializeTuple for ValidatorCompound {
    type Ok = ();
    type Error = FiniteValidationError;

    fn serialize_element<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(FiniteValidator)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl serde::ser::SerializeTupleStruct for ValidatorCompound {
    type Ok = ();
    type Error = FiniteValidationError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(FiniteValidator)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl serde::ser::SerializeTupleVariant for ValidatorCompound {
    type Ok = ();
    type Error = FiniteValidationError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(FiniteValidator)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl serde::ser::SerializeMap for ValidatorCompound {
    type Ok = ();
    type Error = FiniteValidationError;

    fn serialize_key<T: ?Sized + Serialize>(
        &mut self,
        key: &T,
    ) -> Result<(), Self::Error> {
        key.serialize(FiniteValidator)
    }

    fn serialize_value<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(FiniteValidator)
    }

    fn serialize_entry<K: ?Sized + Serialize, V: ?Sized + Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error> {
        key.serialize(FiniteValidator)?;
        value.serialize(FiniteValidator)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl serde::ser::SerializeStruct for ValidatorCompound {
    type Ok = ();
    type Error = FiniteValidationError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(FiniteValidator)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl serde::ser::SerializeStructVariant for ValidatorCompound {
    type Ok = ();
    type Error = FiniteValidationError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(FiniteValidator)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl serde::Serializer for FiniteValidator {
    type Ok = ();
    type Error = FiniteValidationError;
    type SerializeSeq = ValidatorCompound;
    type SerializeTuple = ValidatorCompound;
    type SerializeTupleStruct = ValidatorCompound;
    type SerializeTupleVariant = ValidatorCompound;
    type SerializeMap = ValidatorCompound;
    type SerializeStruct = ValidatorCompound;
    type SerializeStructVariant = ValidatorCompound;

    fn serialize_bool(self, _value: bool) -> Result<(), Self::Error> { Ok(()) }
    fn serialize_i8(self, _value: i8) -> Result<(), Self::Error> { Ok(()) }
    fn serialize_i16(self, _value: i16) -> Result<(), Self::Error> { Ok(()) }
    fn serialize_i32(self, _value: i32) -> Result<(), Self::Error> { Ok(()) }
    fn serialize_i64(self, _value: i64) -> Result<(), Self::Error> { Ok(()) }
    fn serialize_i128(self, _value: i128) -> Result<(), Self::Error> { Ok(()) }
    fn serialize_u8(self, _value: u8) -> Result<(), Self::Error> { Ok(()) }
    fn serialize_u16(self, _value: u16) -> Result<(), Self::Error> { Ok(()) }
    fn serialize_u32(self, _value: u32) -> Result<(), Self::Error> { Ok(()) }
    fn serialize_u64(self, _value: u64) -> Result<(), Self::Error> { Ok(()) }
    fn serialize_u128(self, _value: u128) -> Result<(), Self::Error> { Ok(()) }

    fn serialize_f32(self, value: f32) -> Result<(), Self::Error> {
        if value.is_finite() { Ok(()) } else { Err(FiniteValidationError) }
    }

    fn serialize_f64(self, value: f64) -> Result<(), Self::Error> {
        if value.is_finite() { Ok(()) } else { Err(FiniteValidationError) }
    }

    fn serialize_char(self, _value: char) -> Result<(), Self::Error> { Ok(()) }
    fn serialize_str(self, _value: &str) -> Result<(), Self::Error> { Ok(()) }
    fn serialize_bytes(self, _value: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    fn serialize_none(self) -> Result<(), Self::Error> { Ok(()) }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<(), Self::Error> { Ok(()) }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Self::Error> { Ok(()) }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(ValidatorCompound)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(ValidatorCompound)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(ValidatorCompound)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(ValidatorCompound)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(ValidatorCompound)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(ValidatorCompound)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(ValidatorCompound)
    }

    fn collect_str<T: ?Sized + std::fmt::Display>(self, _value: &T) -> Result<(), Self::Error> {
        Ok(())
    }

    fn is_human_readable(&self) -> bool { true }
}

fn canonical_payload_bytes<T: Serialize>(
    payload: &ReplayPayload<T>,
) -> Result<Vec<u8>, ReplayIntegrityError> {
    if payload.format_version != REPLAY_FORMAT_VERSION
        || payload.contract_version != CAMERA_CONTRACT_VERSION
    {
        return Err(ReplayIntegrityError::VersionMismatch);
    }
    payload
        .serialize(FiniteValidator)
        .map_err(|_| ReplayIntegrityError::Serialization)?;
    let value = serde_json::to_value(payload).map_err(|_| ReplayIntegrityError::Serialization)?;
    let mut bytes = Vec::with_capacity(256);
    bytes.extend_from_slice(b"DIVINE-CAMERA-REPLAY\0");
    write_canonical_json(&value, &mut bytes)?;
    Ok(bytes)
}

pub fn seal_replay<T: Serialize>(
    payload: &ReplayPayload<T>,
) -> Result<ReplaySeal, ReplayIntegrityError> {
    let bytes = canonical_payload_bytes(payload)?;
    Ok(ReplaySeal {
        algorithm: SEAL_ALGORITHM.into(),
        digest_hex: blake3::hash(&bytes).to_hex().to_string(),
    })
}

pub fn verify_replay<T: Serialize>(
    payload: &ReplayPayload<T>,
    seal: &ReplaySeal,
) -> Result<(), ReplayIntegrityError> {
    if seal.algorithm != SEAL_ALGORITHM {
        return Err(ReplayIntegrityError::SealMismatch);
    }
    let actual = seal_replay(payload)?;
    if seal.digest_hex != actual.digest_hex {
        return Err(ReplayIntegrityError::SealMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn playback_uses_single_ordered_stream_and_checks_consumption() {
        let p = DVec3::new(1.0, 2.0, 3.0);
        let events = vec![
            WorldQuery::SampleSurface {
                position: p,
                result: None,
            },
            WorldQuery::ResolveSegment {
                origin: p,
                desired: p + DVec3::X,
                radius: 0.5,
                result: p,
            },
        ];
        let mut world = RecordedWorldPlayback::new(events);
        assert_eq!(world.sample_surface(p).unwrap(), None);
        assert_eq!(
            world.resolve_camera_segment(p, p + DVec3::X, 0.5).unwrap(),
            p
        );
        assert_eq!(world.finish_step(), Ok(()));
    }

    #[test]
    fn playback_rejects_reordered_or_wrong_inputs() {
        let p = DVec3::ZERO;
        let mut world = RecordedWorldPlayback::new(vec![WorldQuery::SampleSurface {
            position: p,
            result: None,
        }]);
        assert!(matches!(
            world.sample_surface(DVec3::ONE),
            Err(WorldQueryError::Diverged { index: 0 })
        ));
    }

    #[test]
    fn rejects_non_finite_query_inputs_without_consuming_event() {
        let mut world = RecordedWorldPlayback::new(vec![WorldQuery::SampleSurface {
            position: DVec3::ZERO,
            result: None,
        }]);
        assert_eq!(
            world.sample_surface(DVec3::splat(f64::NAN)),
            Err(WorldQueryError::InvalidInput)
        );
        assert_eq!(world.consumed(), 0);
    }

    #[test]
    fn rejects_invalid_recorded_surface_normal() {
        let events = vec![WorldQuery::SampleSurface {
            position: DVec3::ZERO,
            result: Some(SurfaceSample {
                position: DVec3::ZERO,
                normal: Vec3::ZERO,
                surface_id: None,
            }),
        }];
        assert_eq!(
            RecordedWorldPlayback::try_new(events).unwrap_err(),
            WorldQueryError::InvalidInput
        );
    }

    #[test]
    fn rejects_terrain_hit_that_does_not_lie_on_ray() {
        let ray = WorldRay {
            origin: DVec3::ZERO,
            direction: Vec3::X,
            min_distance: 0.0,
            max_distance: 100.0,
        };
        let hit = TerrainHit {
            position: DVec3::new(10.0, 1.0, 0.0),
            normal: Vec3::Y,
            distance: 10.0,
            surface_id: None,
        };
        assert_eq!(
            validate_terrain_hit(&ray, &hit),
            Err(WorldQueryError::InvalidInput)
        );
    }

    #[test]
    fn rejects_ray_with_non_unit_direction() {
        let ray = WorldRay {
            origin: DVec3::ZERO,
            direction: Vec3::new(2.0, 0.0, 0.0),
            min_distance: 0.0,
            max_distance: 100.0,
        };
        assert_eq!(validate_ray(&ray), Err(WorldQueryError::InvalidInput));
    }

    #[test]
    fn canonical_seal_is_independent_of_map_insertion_order() {
        let mut first = HashMap::new();
        first.insert("z".to_string(), 1u32);
        first.insert("a".to_string(), 2u32);
        let mut second = HashMap::new();
        second.insert("a".to_string(), 2u32);
        second.insert("z".to_string(), 1u32);
        let p1 = ReplayPayload {
            format_version: REPLAY_FORMAT_VERSION,
            contract_version: CAMERA_CONTRACT_VERSION,
            payload: first,
        };
        let p2 = ReplayPayload {
            format_version: REPLAY_FORMAT_VERSION,
            contract_version: CAMERA_CONTRACT_VERSION,
            payload: second,
        };
        assert_eq!(seal_replay(&p1).unwrap(), seal_replay(&p2).unwrap());
    }

    #[test]
    fn tampering_invalidates_seal_before_replay() {
        let mut payload = ReplayPayload {
            format_version: REPLAY_FORMAT_VERSION,
            contract_version: CAMERA_CONTRACT_VERSION,
            payload: vec![1u32, 2],
        };
        let seal = seal_replay(&payload).unwrap();
        verify_replay(&payload, &seal).unwrap();
        payload.payload[0] = 9;
        assert_eq!(
            verify_replay(&payload, &seal),
            Err(ReplayIntegrityError::SealMismatch)
        );
    }

    #[test]
    fn rejects_non_finite_payload_numbers_instead_of_coercing_to_null() {
        let payload = ReplayPayload {
            format_version: REPLAY_FORMAT_VERSION,
            contract_version: CAMERA_CONTRACT_VERSION,
            payload: f64::NAN,
        };
        assert_eq!(
            seal_replay(&payload),
            Err(ReplayIntegrityError::Serialization)
        );
    }

    #[test]
    fn rejects_unsupported_replay_version() {
        let payload = ReplayPayload {
            format_version: 99,
            contract_version: CAMERA_CONTRACT_VERSION,
            payload: vec![1u32],
        };
        assert_eq!(
            seal_replay(&payload),
            Err(ReplayIntegrityError::VersionMismatch)
        );
    }
}
