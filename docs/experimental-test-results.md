# Experimental AMD descriptor-class guard: 2026-11-25

This report is a local, self-reported test result, not independent verification.
The release is not stable, and no prologue completion is claimed.

## Change

Scalar loads with a computed descriptor index previously treated zero-stride
entries as constant buffers, including textures and missing entries. Descriptor
metadata now records SRV/UAV/CBV classes. Scalar loads only read valid buffer
SRVs or CBVs; other classes return zero. Vector indexing masks class bits out
of the stored stride. This guards one invalid-access path, not all GPU faults.

## Validation

- `cargo test -p victory-shader --features host -- --test-threads=1 --nocapture`:
  211 passed, 0 failed, 2 ignored. The new scalar-heap regression exercised
  valid SRVs and CBVs, textures, UAVs, missing and out-of-range descriptors on
  the hardware adapter and WARP.
- The host descriptor-class metadata unit test passed (1 test).
- `cargo build --profile diag -p victory --features diagnostics` passed.
- Two locally investigated shaders passed DXIL validation. No extracted shaders
  or other title-derived files are distributed.
- Asset-policy checks and `git diff --check` passed.

## Long run

Windows 11 x64, RX 9070 XT 16 GB, 32 GB system RAM. The tested diagnostics
executable SHA-256 is
`E8F3961FBFF4F60D852904433C84D1E3BA4D2E5483A547D28B510F4445200C44`.

The run started at 04:10:42 and exited at 04:53:21 (UTC+03:00) on November 25,
2026. At the 42.3-minute checkpoint it had passed frame 23,000 with compute
shader `0452fc2d2e2fc4c3` enabled and no fatal error recorded so far. Shortly
afterward the runtime reported GPU device removal and a page fault at
`0x251667000`. The stopped command was an indirect mesh dispatch (mesh
`0a2ea81ca59d6510`, pixel `19acd330dacc8f32`). This later failure supersedes
any interpretation of the checkpoint as a completed crash-free test.

The launcher uses Wave32, synchronous submission, DRED, reduced pipeline/copy
logging, and 31 pipeline skips. Skipped passes can affect rendering. Menus
were approximately 30 fps; 3D scenes varied around 0.7–4 fps, with missing
geometry/textures and dark regions. Gameplay control and prologue completion
were not verified. Additional GPU-fault investigation is still required.

## Distribution

Only the open-source runtime, launcher, license, instructions, and checksums
are packaged. No title executable, assets, shader dumps, local configuration,
captures, crash dumps, or raw test logs are included. Full upstream history
and GPL-3.0-or-later credits remain in the public source repository.
