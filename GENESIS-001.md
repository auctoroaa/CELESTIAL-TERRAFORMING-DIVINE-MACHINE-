# Genesis-001: verified planetary construction

Pipeline: Intent validation -> causal-plan resolution -> deterministic spherical terrain construction -> independent geometric verification -> planetary-state commit.

Acceptance fixture Aster: seed 471029; radius 6,000,000 m; gravity 9.8 m/s2; resolution 128; elevation scale 8,500 m; noise frequency 0.004.

Guarantees:
- Invalid and non-finite intent values are rejected before construction.
- Identical inputs produce deterministic geometry and digests on the same supported Rust target.
- The verifier independently checks cardinality, index bounds, finite coordinates, radial bounds, unit normals, and SHA-256 integrity.
- A revision-1 planetary state is emitted only after verification passes.
- Output is a constructed geometric proposal with evidence, not a claim that physical planetary laws or real terraforming have been established.

Run cargo test, cargo run --release, or cargo run --release -- path/to/intent.json.
The CLI prints the complete result as JSON. With no argument it runs the Aster fixture.
