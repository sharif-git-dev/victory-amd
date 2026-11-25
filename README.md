# victory-amd

AMD RDNA 4 / Tier-3 Direct3D 12 stability and performance fork of
[fxceless/victory-fork](https://github.com/fxceless/victory-fork) and
[Marvisak/VIctory](https://github.com/Marvisak/VIctory): an experimental Windows
x86-64 compatibility layer (`xgameruntime` / `d3d12_xs` HLE runtime and RDNA 2 →
DXIL shader recompiler) that executes user-supplied Xbox Series binaries natively
on PC.

> **No proprietary files:** This repository contains **zero** game files, assets,
> keys, shader dumps, or title code. Every source file is checked by
> `cargo xtask policy`. You must supply your own legal dump of a title outside
> the repository tree.

---

## Latest experimental build — 2026-11-25

[Download v0.1.0-amd.1](https://github.com/sharif-git-dev/victory-amd/releases/tag/v0.1.0-amd.1)
contains the tested Windows x64 **diagnostics** runtime and an AMD launcher.
This is an experimental build: the latest long run **ended in a GPU page fault
after approximately 42 minutes 39 seconds**. It is not a stable or fully playable release.

The new descriptor-class guard prevents scalar heap loads from interpreting
texture, UAV, or missing descriptors as constant buffers. Regression tests cover
buffer SRVs, CBVs, invalid descriptor classes, and out-of-range indices on both
hardware and WARP. The shader suite passed **211 tests, with 2 ignored**; the
host descriptor-metadata test also passed. These are the checks for this update;
the older full-workspace results below belong to the previous release.

The run passed frame 23,000 with the previously implicated compute shader
enabled, then faulted during a different indirect mesh dispatch. The launcher
still skips 31 known pipelines; rendering is incomplete, dark, and typically
around 0.7–4 fps in 3D scenes. Prologue completion has not been verified.
See [test results and limitations](docs/experimental-test-results.md).

---

## Credits & Upstream Lineage

- **[Marvisak/VIctory](https://github.com/Marvisak/VIctory)** — foundational PE
  mapper, `xgameruntime` / GDK high-level runtime, Direct3D 12 translation back
  end, and RDNA 2 → DXIL shader compiler (`GPL-3.0-or-later`).
- **[fxceless/victory-fork](https://github.com/fxceless/victory-fork)** (`hyphennn`) —
  world meshlet culling fixes, descriptor table sizing, compute lighting
  read-backs, and sunlit prologue rendering (`GPL-3.0-or-later`).
- **[ivall/xcl](https://github.com/ivall/xcl)** — architectural reference credited
  by upstream.

Full upstream commit history is preserved intact under `GPL-3.0-or-later`.

---

## What `victory-amd` Fixes on AMD RDNA 4 (RX 9070 XT)

Upstream `victory-fork` was developed and tuned on an NVIDIA RTX 4070 (`WaveLaneCountMax = 32`,
Resource Binding Tier 2 view heap capped at `1,000,000` descriptors). Running unmodified
on an **AMD Radeon RX 9070 XT** (RDNA 4, `WaveLaneCountMin = 32` / `WaveLaneCountMax = 64`,
Tier 3 unbounded descriptor heaps) hit three distinct failure modes during loading and the
opening sequence. This fork adds mitigations for those failures; further GPU
faults remain, as documented above:

### 1. Tier-3 Bindless Descriptor Heap Bounds Clamping (`crates/victory-gpu/src/descriptor/host.rs`)
- **Root cause (caught via DRED):** NVIDIA caps shader-visible `CBV_SRV_UAV` heaps at
  `1,000,000` descriptors, causing `probe_view_limit()` to fail the `1,000,001` probe and
  emit `UMin(index, 999_999)` onto a trailing null descriptor slot. On Tier-3 AMD RDNA 4
  GPUs, `CreateDescriptorHeap(1_000_001)` succeeds, so `VIEW_LIMIT` remained `0` (unclamped)
  and the heap was sized to exact `count + DYNAMIC` (`1,507,328` slots) with no trailing
  null guard. When indirect compute dispatch `1f1fbd86759b3da2` indexed garbage bindless
  slot `105,425,664`, RDNA 4 GPU-page-faulted at `0x1c9246000` (`0xc6356000` past the heap)
  and triggered Windows TDR (`LiveKernelEvent 141` / `0x887A0006`).
- **Fix:** Tier-3 hosts now allocate `2,097,152` (`SLOT_STRIDES_COUNT`, `1 << 21`) null-initialized
  view slots (preserving all `1,507,328` title + dynamic slots without truncation while keeping
  trailing slots null), pad shader-visible sampler heaps to `2,048` slots to match `UMin(s, 2047)`,
  set `set_view_heap_limit(2_097_151)` before first shader translation, and include
  `view_heap_limit()` in the shader cache build fingerprint (`crates/victory-gpu/src/shader_cache.rs`).

### 2. Automatic MSVC SEH Guard Around Driver PSO Compilation (`crates/victory-gpu/src/seh.c`, `pipeline/host.rs`)
- **Root cause:** Around 30–41 compute shaders (e.g. `CS_RaytracingResolve` ped material
  variants) trigger a null-pointer read (`0xc0000005` reading `0x8`) inside AMD's driver
  compiler (`amdxc64.dll`) during `CreateComputePipelineState`, terminating the entire
  process unless manually bisected one-by-one into `VICTORY_SKIP_PIPELINES`.
- **Fix:** All host D3D12 PSO creation calls (`CreateComputePipelineState`,
  `CreateGraphicsPipelineState`, and mesh `CreatePipelineState`) run inside a lightweight
  MSVC `__try / __except` wrapper (`seh_pso`). Any `amdxc64.dll` access violation is caught
  at the FFI boundary, logged with its 16-digit shader hash, and converted into a clean
  `E_FAIL` pipeline refusal so the title continues running automatically without manual
  offline shader bisection.

### 3. Runtime `VICTORY_HOST_MAX_WAVE` Override & Runtime DXIL Validation (`host.rs`, `shader_cache.rs`)
- **Fix:** Wired `VICTORY_HOST_MAX_WAVE=32` into live `victory run` initialization (and into
  the `shader_cache` key) so dual-wave RDNA 4 GPUs (`min=32, max=64`) can target the
  battle-tested 32-lane lowering path at native hardware speed, alongside automatic
  `dxil.dll` validation on shader cache misses (`VICTORY_VALIDATE_DXIL=0` to disable).

### 4. Synchronous Submission Stability vs. `VICTORY_RELAY=2` on RDNA 4
- Controlled A/B DRED runs on RX 9070 XT showed that queuing 4 submissions ahead with
  `VICTORY_RELAY=2` triggers an in-flight buffer unmap page fault (`0x227b000`) at ~15.5
  minutes, whereas synchronous submission (`VICTORY_RELAY` unset) runs the exact same
  indirect compute passes every frame across **22+ minute continuous sessions with zero
  device removals**.

---

## Historical measurements: v0.1.0-amd

**Test system:** Windows 11 x64 (build 26300), AMD Radeon RX 9070 XT 16 GB (RDNA 4),
32 GB RAM, Xbox Series X profile.

| Milestone | Observed Result on RX 9070 XT |
|-----------|-------------------------------|
| Workspace test suite (`cargo test --workspace`) | **497 passed, 0 failed** (including SEH access-violation unit test) |
| Boot, setup screens, profile saves, start menu | **30.0 fps**, 0 crashes |
| Lit opening cutscene & credits sequence | **3.0–3.6 fps** (`278–365 ms/frame`, peaks up to `4.6 fps`), lit sky, vehicles, characters, volumetric lights |
| Continuous stability (synchronous mode) | **22+ minutes (`1,329.7 s`) continuous run, 0 crashes, 0 TDR / device removals**, `6,841` cached shaders/pipelines |
| Known upstream limits | Shadowed areas lacking ray-traced bounce/sun shadow passes remain dark; world framerate is CPU/copy bound at ~3–4 fps |

---

## Build & Run

Requirements: Windows 10/11 x64, Rust `1.98.1` (`rust-toolchain.toml`), Visual Studio
C++ Build Tools (MSVC), Direct3D 12 GPU with Shader Model 6.6 + mesh shaders.

```powershell
cargo xtask policy                                              # verify zero asset-policy violations
copy victory.example.toml victory.toml                          # point paths OUTSIDE the repository
cargo test --workspace                                          # run unit tests (no title files needed)
cargo build --release -p victory                                # target\release\victory.exe
cargo build --profile diag -p victory --features diagnostics    # target\diag\victory.exe (DRED + diagnostics)
```

Launch with the experimental AMD RDNA 4 preset (the script keeps its historical filename):

```powershell
scripts\launch-amd-stable.cmd "D:\your-title-folder\title.exe"
```

The latest [prerelease](https://github.com/sharif-git-dev/victory-amd/releases/tag/v0.1.0-amd.1)
ships only the tested diagnostics runtime, compiled from this open-source
repository with zero title data. Extract the ZIP and run
`launch-amd.cmd "D:\your-title-folder\title.exe"`. The corresponding source is
available at the release tag. You must supply your own title files separately.

---

## License

Licensed under **GPL-3.0-or-later**. See [LICENSE](LICENSE).
