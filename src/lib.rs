use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenesisIntent {
    pub designation: String, pub seed: u64, pub radius_m: f64, pub gravity_m_s2: f64,
    pub terrain_resolution: u32, pub elevation_scale_m: f64, pub noise_frequency: f64,
}
impl GenesisIntent {
    pub fn aster() -> Self { Self { designation:"Aster".into(), seed:471029, radius_m:6_000_000.0,
        gravity_m_s2:9.8, terrain_resolution:128, elevation_scale_m:8500.0, noise_frequency:0.004 } }
    pub fn validate(&self) -> Result<(), EngineError> {
        if self.designation.trim().is_empty(){return Err(EngineError::InvalidIntent("designation is empty"));}
        if !self.radius_m.is_finite()||self.radius_m<=0.0{return Err(EngineError::InvalidIntent("radius must be finite and positive"));}
        if !self.gravity_m_s2.is_finite()||self.gravity_m_s2<0.0{return Err(EngineError::InvalidIntent("gravity must be finite and non-negative"));}
        if !(8..=1024).contains(&self.terrain_resolution){return Err(EngineError::InvalidIntent("resolution must be in 8..=1024"));}
        if !self.elevation_scale_m.is_finite()||self.elevation_scale_m<0.0||self.elevation_scale_m>=self.radius_m{return Err(EngineError::InvalidIntent("elevation scale is outside valid bounds"));}
        if !self.noise_frequency.is_finite()||self.noise_frequency<=0.0{return Err(EngineError::InvalidIntent("noise frequency must be finite and positive"));}
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CausalPlan { pub source_intent_digest:String, pub resolution:u32, pub seed:u64,
    pub base_radius_m:f64, pub elevation_scale_m:f64, pub noise_frequency:f64, pub construction_rule:String }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanetMesh { pub positions:Vec<[f64;3]>, pub normals:Vec<[f64;3]>, pub indices:Vec<u32>,
    pub digest:String, pub min_radius_m:f64, pub max_radius_m:f64 }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationEvidence { pub accepted:bool, pub checks:Vec<String>, pub vertex_count:usize,
    pub triangle_count:usize, pub mesh_digest:String, pub evidence_digest:String }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanetaryState { pub designation:String, pub revision:u64, pub intent_digest:String,
    pub mesh_digest:String, pub evidence_digest:String, pub vertex_count:usize, pub triangle_count:usize }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenesisResult { pub intent:GenesisIntent, pub causal_plan:CausalPlan, pub mesh:PlanetMesh,
    pub evidence:VerificationEvidence, pub committed_state:PlanetaryState }
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("invalid intent: {0}")] InvalidIntent(&'static str),
    #[error("construction failed: {0}")] Construction(&'static str),
    #[error("verification failed: {0}")] Verification(&'static str),
    #[error("serialization failed")] Serialization,
}
fn digest(b:&[u8])->String { let mut h=Sha256::new();h.update(b);h.finalize().iter().map(|x|format!("{x:02x}")).collect() }
pub fn resolve_causality(i:&GenesisIntent)->Result<CausalPlan,EngineError>{
    i.validate()?;let bytes=serde_json::to_vec(i).map_err(|_|EngineError::Serialization)?;
    Ok(CausalPlan{source_intent_digest:digest(&bytes),resolution:i.terrain_resolution,seed:i.seed,
        base_radius_m:i.radius_m,elevation_scale_m:i.elevation_scale_m,noise_frequency:i.noise_frequency,
        construction_rule:"spherical-latitude-longitude-heightfield-v1".into()})
}
fn lattice(x:i64,y:i64,s:u64)->f64{let mut z=(x as u64).wrapping_mul(0x9E3779B185EBCA87)^(y as u64).wrapping_mul(0xC2B2AE3D27D4EB4F)^s;
 z^=z>>30;z=z.wrapping_mul(0xBF58476D1CE4E5B9);z^=z>>27;z=z.wrapping_mul(0x94D049BB133111EB);z^=z>>31;(z as f64/u64::MAX as f64)*2.0-1.0}
fn smooth(t:f64)->f64{t*t*(3.0-2.0*t)}
fn noise(x:f64,y:f64,s:u64)->f64{let(ix,iy)=(x.floor()as i64,y.floor()as i64);let(fx,fy)=(smooth(x-ix as f64),smooth(y-iy as f64));
 let(a,b,c,d)=(lattice(ix,iy,s),lattice(ix+1,iy,s),lattice(ix,iy+1,s),lattice(ix+1,iy+1,s));
 let ab=a+(b-a)*fx;let cd=c+(d-c)*fx;ab+(cd-ab)*fy}
fn height(lat:f64,lon:f64,p:&CausalPlan)->f64{let(x,y,z)=(lon.cos()*lat.cos(),lat.sin(),lon.sin()*lat.cos());let f=p.noise_frequency*p.base_radius_m;
 p.elevation_scale_m*(0.78*noise(x*f,z*f,p.seed)+0.22*noise(x*f*2.03+17.0,(y+z)*f*2.03-9.0,p.seed^0xA5A5A5A5))}
fn mesh_hash(m:&PlanetMesh)->String{let mut b=Vec::new();for p in m.positions.iter().chain(m.normals.iter()){for v in p{b.extend_from_slice(&v.to_bits().to_le_bytes());}}
 for i in &m.indices{b.extend_from_slice(&i.to_le_bytes());}digest(&b)}
pub fn construct(p:&CausalPlan)->Result<PlanetMesh,EngineError>{
 let n=p.resolution as usize;let count=n.checked_mul(n).ok_or(EngineError::Construction("vertex overflow"))?;
 if count>u32::MAX as usize{return Err(EngineError::Construction("index range exceeded"));}
 let mut positions=Vec::with_capacity(count);let mut normals=Vec::with_capacity(count);let(mut min_r,mut max_r)=(f64::INFINITY,0.0_f64);
 for iy in 0..n{let lat=-std::f64::consts::FRAC_PI_2+std::f64::consts::PI*iy as f64/(n-1)as f64;
  for ix in 0..n{let lon=-std::f64::consts::PI+std::f64::consts::TAU*ix as f64/(n-1)as f64;let r=p.base_radius_m+height(lat,lon,p);
   min_r=min_r.min(r);max_r=max_r.max(r);positions.push([r*lat.cos()*lon.cos(),r*lat.sin(),r*lat.cos()*lon.sin()]);
   normals.push([lat.cos()*lon.cos(),lat.sin(),lat.cos()*lon.sin()]);}}
 let mut indices=Vec::with_capacity((n-1)*(n-1)*6);for y in 0..n-1{for x in 0..n-1{let a=(y*n+x)as u32;let b=a+1;let c=a+n as u32;let d=c+1;indices.extend_from_slice(&[a,c,b,b,c,d]);}}
 let mut m=PlanetMesh{positions,normals,indices,digest:String::new(),min_radius_m:min_r,max_radius_m:max_r};m.digest=mesh_hash(&m);Ok(m)
}
/// Independent structural and integrity verifier. Does not invoke the constructor.
pub fn verify(p:&CausalPlan,m:&PlanetMesh)->VerificationEvidence{
 let n=p.resolution as usize;let(mut ok,mut checks)=(true,Vec::new());
 macro_rules! check{($cond:expr,$name:expr)=>{if $cond{checks.push($name.into())}else{ok=false}}}
 check!(m.positions.len()==n*n&&m.normals.len()==n*n,"vertex-and-normal-cardinality");
 check!(m.indices.len()==(n-1)*(n-1)*6&&m.indices.len()%3==0,"triangle-cardinality");
 check!(m.indices.iter().all(|i|(*i as usize)<m.positions.len()),"indices-in-bounds");
 check!(m.positions.iter().flatten().chain(m.normals.iter().flatten()).all(|v|v.is_finite()),"finite-geometry");
 check!(m.positions.iter().all(|v|{let r=(v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt();r>=p.base_radius_m-p.elevation_scale_m*1.01&&r<=p.base_radius_m+p.elevation_scale_m*1.01}),"radial-bounds");
 check!(m.normals.iter().all(|v|((v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt()-1.0).abs()<1e-9),"unit-normals");
 let mut bytes=Vec::new();for v in m.positions.iter().chain(m.normals.iter()){for x in v{bytes.extend_from_slice(&x.to_bits().to_le_bytes());}}for i in &m.indices{bytes.extend_from_slice(&i.to_le_bytes());}
 let independent=digest(&bytes);check!(independent==m.digest,"independent-sha256-match");
 let ev=serde_json::to_vec(&(ok,&checks,m.positions.len(),m.indices.len()/3,&independent)).unwrap_or_default();
 VerificationEvidence{accepted:ok,checks,vertex_count:m.positions.len(),triangle_count:m.indices.len()/3,mesh_digest:independent.clone(),evidence_digest:digest(&ev)}
}
pub fn genesis(intent:GenesisIntent)->Result<GenesisResult,EngineError>{
 intent.validate()?;let plan=resolve_causality(&intent)?;let mesh=construct(&plan)?;let evidence=verify(&plan,&mesh);
 if !evidence.accepted{return Err(EngineError::Verification("independent checks rejected artifact"));}
 // State revision is created only after verification succeeds.
 let committed_state=PlanetaryState{designation:intent.designation.clone(),revision:1,intent_digest:plan.source_intent_digest.clone(),
  mesh_digest:mesh.digest.clone(),evidence_digest:evidence.evidence_digest.clone(),vertex_count:evidence.vertex_count,triangle_count:evidence.triangle_count};
 Ok(GenesisResult{intent,causal_plan:plan,mesh,evidence,committed_state})
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn aster_genesis_verifies(){let r=genesis(GenesisIntent::aster()).unwrap();assert!(r.evidence.accepted);assert_eq!(r.committed_state.revision,1);assert_eq!(r.evidence.vertex_count,16384);assert_eq!(r.evidence.triangle_count,32258);}
 #[test]fn same_seed_is_deterministic(){let a=genesis(GenesisIntent::aster()).unwrap();let b=genesis(GenesisIntent::aster()).unwrap();assert_eq!(a.mesh.digest,b.mesh.digest);assert_eq!(a.mesh.positions,b.mesh.positions);}
 #[test]fn invalid_input_is_rejected(){let mut i=GenesisIntent::aster();i.radius_m=f64::NAN;assert!(genesis(i).is_err());}
 #[test]fn verifier_detects_tampering(){let p=resolve_causality(&GenesisIntent::aster()).unwrap();let mut m=construct(&p).unwrap();m.positions[0][0]+=1.0;assert!(!verify(&p,&m).accepted);}
 #[test]fn seed_changes_geometry(){let a=genesis(GenesisIntent::aster()).unwrap();let mut i=GenesisIntent::aster();i.seed+=1;let b=genesis(i).unwrap();assert_ne!(a.mesh.digest,b.mesh.digest);}
}
