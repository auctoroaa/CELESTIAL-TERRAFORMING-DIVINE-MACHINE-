# Celestial Terraforming Divine Engine
## Project Guide for Future ChatGPT Agents and Contributors

> **What we are building:** A universal creation engine that turns a creator's intent into designed, constructed, simulated, and verifiably described results. It must support creations at any represented scale, from a small component or artifact to machines, structures, organisms, ecosystems, environments, complete worlds, planets, and larger celestial systems.

This document is the project-orientation guide. Read it before proposing architecture, changing code, creating milestones, or narrowing the project's purpose.

## 1. Product identity

The Celestial Terraforming Divine Engine is **not merely a planet generator, terrain generator, camera, renderer, physics simulator, or procedural-world demo**.

It is intended to be a general-purpose construction and creation engine. Planetary creation and terraforming are important application domains, but they do not define the limits of the engine.

A planet is a complete world-building target, not simply a spherical mesh. Depending on the creator's intent, a world may include geometry, materials, internal structure, environments, governing laws, processes, ecosystems, entities, relationships, history, and behavior.

The engine must also be useful when the requested result is not a world at all: for example, a component, artifact, machine, building, organism, material system, or other construct.

**Universal core, specialized capabilities, many construction domains.** Do not create a separate foundational engine for every domain.

## 2. The creator's intent is the authority

The creator defines **what should exist and what properties, relationships, rules, and behaviors it should have**. The engine determines and executes a technically valid way to realize that intent.

The engine must preserve the distinction between:
- **Declared:** explicitly requested or supplied by the creator.
- **Derived:** logically or computationally inferred from declared information.
- **Assumed:** supplied as a temporary default and clearly identified as such.
- **Unknown:** not yet specified or established.
- **Conflicting:** incompatible requirements or laws requiring resolution.

Never silently convert an assumption into creator intent. Never invent a requirement and report it as user-approved. Unknown is a valid state. Preserve source intent and its revisions.

The engine should support different governing laws and rule systems. Do not automatically impose Earth's physics, chemistry, biology, units, or environmental assumptions on a constructed realm. Apply the laws declared for the target. Identify conflicts relative to those laws, and only compare against conventional Earth science when requested or relevant to an explicit compatibility requirement.

## 3. Universal creation lifecycle

The intended high-level lifecycle is:

1. **Divine Intent:** Capture what the creator wants, including objectives, targets, requirements, constraints, preferences, governing laws, and acceptance criteria.
2. **Intent Formalization:** Convert the source into a structured, versioned representation without losing provenance or uncertainty.
3. **Design and Causal Planning:** Determine entities, properties, relationships, dependencies, construction order, and required operations.
4. **Capability Resolution:** Discover and compose the domain capabilities required for the requested result. Check availability, compatibility, dependencies, and limitations.
5. **Construction:** Execute typed, traceable construction operations and produce actual artifacts or state changes.
6. **Simulation and Evaluation:** Exercise behavior and interactions where the intent requires them.
7. **Verification:** Compare the actual result against the original intent, declared laws, constraints, and acceptance criteria.
8. **Artifact and History:** Preserve outputs, versions, inputs, lineage, execution records, and evidence so results can be inspected and reproduced where deterministic execution is supported.

In short:

**Creator Intent → Formal Specification → Design → Capability Resolution → Construction → Simulation → Verification → Artifact**

A successful execution is not automatically a fulfilled intent. Verification must evaluate the result, not merely whether commands completed.

## 4. Universal core and capability domains

The core should operate on general concepts: entities, types, properties, relationships, spatial and temporal structure, rules, behaviors, constraints, dependencies, operations, state, evidence, and provenance.

Specialized capabilities provide domain knowledge and operations. Potential domains include:

- Geometry, topology, spatial representation, and scale.
- Materials, composition, and material behavior.
- Physical and alternative governing-law systems.
- Chemistry and transformation processes.
- Biology, organisms, and ecosystems.
- Architecture, structures, and infrastructure.
- Machines, mechanisms, and engineered artifacts.
- Environment, climate, atmosphere, hydrology, and geology.
- Planetary formation, planetary transformation, and terraforming.
- Celestial mechanics and larger astronomical constructions.
- Simulation, optimization, validation, and verification.

This list is illustrative, not a closed registry. The engine should allow new domains and user-defined capabilities without redesigning the universal core.

Not every construction requires every capability. The resolver should select only what the intent needs. Missing capabilities must be reported honestly; do not pretend an unsupported construction was completed.

## 5. Divine Intent implementation direction

Divine Intent is the universal entry point and intent-preservation system, not a planet-specific form.

Its eventual responsibilities include:
- Accepting natural-language, structured, visual, and referenced inputs.
- Preserving the original source and creator identity/authority where applicable.
- Extracting target entities, properties, relationships, behaviors, rules, constraints, and acceptance criteria.
- Representing uncertainty, assumptions, omissions, and contradictions explicitly.
- Compiling validated intent into a versioned, domain-independent representation (working name: **DIR, Divine Intent Representation**).
- Resolving required capabilities and exposing missing dependencies.
- Producing a design/planning input without silently changing the creator's request.
- Linking final verification evidence to individual requirements and the original intent revision.

The DIR name and exact schema are design proposals until formally accepted and implemented. Keep parsing, representation, validation, planning, execution, and verification as separable responsibilities. Do not make a language model the authority for whether construction succeeded.

## 6. Engineering principles

1. **Intent fidelity:** The creator's declared objective and constraints remain traceable through the entire lifecycle.
2. **Domain independence:** Core abstractions must not hard-code planets, terrain, or any one construction domain.
3. **Capability composition:** Domain-specific knowledge belongs in capabilities, not in universal-core conditionals.
4. **Scale independence:** Support arbitrary represented scales and units. Do not assume every target is measured in meters or fits a fixed size taxonomy.
5. **Law-system independence:** A target's declared laws govern its construction and validation.
6. **Determinism where possible:** Identical canonical inputs, versions, seeds, and execution conditions should produce identical results for operations declared deterministic. Explicitly identify nondeterministic operations.
7. **Evidence-backed verification:** Report verified, failed, partially fulfilled, unverified, or blocked requirements. Never claim more than the evidence proves.
8. **Provenance and revision:** Preserve source inputs, versions, lineage, and execution history. Do not silently overwrite important state.
9. **Modularity:** Prefer small, typed, testable crates and clear contracts over a monolith.
10. **Honest maturity reporting:** Distinguish intended architecture, proposed design, implemented code, tested behavior, and production-integrated capability.

## 7. What the current repository proves

The repository is an early Rust engineering foundation, not yet the complete universal creation engine.

At the last verified engineering checkpoint, the workspace contained the `divine-camera` crate. Its camera packet, GPU ABI, and replay integrity work had passed the recorded hosted CI checks: formatting, Clippy with warnings denied, and 19 tests. This verifies that camera/replay foundation only. It does not prove that universal intent compilation, general construction, planetary generation, a renderer integration, or an end-to-end world-building pipeline exists.

Do not describe a proposed subsystem as implemented. Before changing code, inspect the current branch, workspace, tests, documentation, and CI. Update this status section when evidence changes.

## 8. Development direction

Build a narrow, working vertical slice that proves the universal architecture before attempting a vast catalogue of domains.

A sensible progression is:

1. Establish the domain-independent intent and requirement model, with explicit provenance and uncertainty.
2. Validate and version the structured representation.
3. Demonstrate that two materially different target types can use the same intent lifecycle.
4. Add capability registration, dependency resolution, and typed operation contracts.
5. Generate a causal construction plan and execute a small real construction.
6. Verify the result against the source intent.
7. Add more capable construction domains, including planetary construction and terraforming.
8. Integrate simulation, rendering, persistence, and larger-scale orchestration when justified by working requirements.

The first demonstration should be small enough to test thoroughly, but it must prove architectural generality. A simple artifact and a small world target are useful contrasting examples. Do not mistake a toy demo for completion of the full product.

## 9. Instructions to future ChatGPT agents

When continuing this project:

- Start by reading this guide and inspecting the actual repository state. Do not rely on an old conversation summary when the current branch or files can be checked.
- Treat this guide as the product-direction baseline. If implementation conflicts with it, identify the conflict and propose the smallest corrective change.
- Do not narrow the engine to planetary terrain, spherical geometry, camera systems, rendering, or simulation.
- Do not expand scope by inventing features the creator has not requested. Universal architecture means extensibility, not implementing every domain at once.
- Preserve existing working code unless evidence demonstrates it must change.
- Separate accepted decisions from proposals, assumptions, unknowns, and implementation facts.
- For repository work, inspect first, make changes on an appropriate feature branch, add regression tests, run formatting, linting, and tests, and report exact commit and CI evidence.
- Never claim deployment, production readiness, end-to-end construction, or verification without corresponding evidence.
- Ask the creator only when a genuinely consequential ambiguity cannot be resolved from project records or code.

## 10. Definition of the intended product

The intended product is one extensible engine through which a creator can express a desired result, have that intent faithfully formalized, resolve the capabilities needed to realize it, construct the result, evaluate its behavior, and verify what was actually achieved.

**We are building a universal creation engine. Planetary terraforming is one of its capabilities and applications, not the boundary of what it can create.**
