# Divine Camera P5: Precision, GPU Packet, and Replay

## Scope
This change introduces a standalone `divine-camera` Rust crate. It does not replace the renderer, mutate planetary state, or claim that a graphics backend has adopted the GPU ABI.

## Precision contract
- Planetary camera positions remain `DVec3` authoritative values.
- Subtract world positions in f64 before converting camera-relative values to f32.
- Split-origin packets use high/low f32 components. Inputs that overflow f32 are rejected.
- Camera-relative values are local render coordinates, not persistent planetary coordinates.

## GPU ABI
`GpuCameraPacket` is C-layout, 16-byte aligned, 336 bytes, and Pod-safe through bytemuck. Matrices are serialized column-major. The consuming shader must use the same matrix convention and metadata enum values. Backend depth compare state and depth clear values remain renderer responsibilities.

## Replay contract
- Replay payloads use project-defined canonical JSON v1: object keys are sorted, strings use JSON escaping, and non-finite floating-point values are rejected before sealing. BLAKE3 is applied with a replay-specific domain prefix.
- The canonicalization contract is versioned and must be implemented identically by any non-Rust verifier; it is not advertised as RFC 8785.
- Format version 1 and camera contract version 2 are required.
- World interactions use one ordered event stream. Kind/order/input mismatches, exhaustion, and unconsumed events fail closed.
- The adapter interface is mutable so the playback cursor is explicit.
- Exact query equality is intentionally strict for recorded playback. Live-world tolerance policy is not yet implemented by this crate.

## Verification status
Unit tests are included for split-origin precision, subtract-before-narrowing, GPU ABI size/alignment/offsets, ordered replay, query divergence, and tamper detection. They have not been executed in the hosted GitHub connector environment. Run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Do not label this implementation production-verified until those commands pass and the renderer shader ABI is integration-tested.
