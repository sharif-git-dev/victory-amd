//! A computed scalar descriptor must not reinterpret a texture/UAV as a CBV.

use super::host::{Dispatch, Host, Root, Stride, View};
use super::{HostShader, SLOT_STRIDE_CBV, SLOT_STRIDE_UAV, SLOT_STRIDES_COUNT, lower, validator};
use crate::native::{
    ComputeIo, HwStage, NativeProgram, ResourceType, RootParam, RootType, StageIo, UsedResource,
};
use crate::{io, ir, resource, structure};

fn scalar_heap_shader() -> HostShader {
    let mov = |dst: u32, src: u32| 0xbe80_0300 | dst << 16 | src;
    let load =
        |dst: u32, base: u32, offset: u32| [0xf408_0000 | dst << 6 | base >> 1, offset << 25];
    let mut words = vec![
        mov(4, 1),                            // preserve the root constant index
        mov(1, 128),                          // high half of the resource heap base
        0x8f00_0000 | 4 << 16 | 133 << 8 | 4, // s_lshl_b32 s4, s4, 5
    ];
    words.extend(load(8, 0, 4)); // descriptor at heap base + s4
    words.extend([0xf420_0000 | 12 << 6 | 4, 125 << 25]); // scalar dword from it
    words.push(mov(3, 128)); // high half of the output table
    words.extend(load(16, 2, 125));
    words.push(0x7e02_020c); // v_mov_b32 v1, s12
    words.extend([0xe070_0000, 0x8004_0100]); // buffer_store_dword v1, off, s[16:19], 0
    words.push(0xbf81_0000);
    let param = |root, user_data, slot| RootParam {
        root,
        table_offset_dwords: 0,
        sgprs: 1,
        base_register: 0,
        user_data,
        slot,
    };
    let native = NativeProgram {
        stage: HwStage::Cs,
        wave64: false,
        vgprs: 2,
        layout: vec![
            param(RootType::ResourceHeap, 0, 0),
            param(RootType::Constants, 1, 0),
            param(RootType::Table, 2, 1),
        ],
        used: vec![UsedResource {
            kind: ResourceType::Uav,
            root: RootType::Table,
            table_offset_dwords: 0,
            lower_bound: 0,
            stride: 0,
            dwords: 4,
            user_data: 2,
            slot: 1,
        }],
        io: StageIo::Cs(ComputeIo {
            rsrc2: 3 << 1,
            threads: [1, 1, 1],
        }),
        code: words.iter().flat_map(|w| w.to_le_bytes()).collect(),
    };
    let p = ir::build(&native.code, native.wave64, native.vgprs).unwrap();
    let st = structure::structure(&p).unwrap();
    let res = resource::track(&p, &native.layout, native.stage, Some(&st)).unwrap();
    let io = io::raise(&p, &native).unwrap();
    let out = lower::lower(&lower::Parts {
        native: &native,
        p: &p,
        st: &st,
        io: &io,
        res: &res,
        inputs: vec![],
        outputs: vec![],
        prims: vec![],
        mesh_stride: 0,
    })
    .unwrap();
    HostShader {
        stage: HwStage::Cs,
        dxil: out.dxil,
        user_data: out.user_data,
        samplers: out.samplers,
        static_samplers: vec![],
        root_cbvs: out.root_cbvs,
        root_srvs: out.root_srvs,
        root_uavs: out.root_uavs,
        root_indexed_views: res.root_indexed_views,
    }
}

#[test]
fn scalar_heap_load_checks_descriptor_class() {
    let _one = super::valu_tests::ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut shader = scalar_heap_shader();
    if let Ok(mut v) = validator::Validator::load(validator::Validator::default_path().as_deref()) {
        v.validate(&mut shader.dxil).unwrap();
    }
    assert!(shader.root_srvs.contains(&255));
    let mut metadata = vec![0; SLOT_STRIDES_COUNT as usize];
    metadata[..5].copy_from_slice(&[
        1,
        SLOT_STRIDE_CBV,
        0,
        SLOT_STRIDE_UAV | 1,
        SLOT_STRIDE_UAV | 1,
    ]);
    let input = [0x1234_5678];
    let constant = [0x8765_4321];
    let pixel = [[1., 2., 3., 4.]];
    let views = [
        View::Buffer {
            data: &input,
            stride: Stride::Raw,
        },
        View::Cbv(&constant),
        View::Texture(1, 1, &pixel),
        View::RawUav(1),
        View::RawUav(1),
    ];
    for (backend, warp) in [("hardware", false), ("warp", true)] {
        let host = match Host::new(warp) {
            Ok(host) => host,
            Err(error) => {
                eprintln!("{backend}: skipped ({error})");
                continue;
            }
        };
        let (root, pso) = host.compute_pipeline(&shader).unwrap();
        for (index, expected) in [
            (0, input[0]),
            (1, constant[0]),
            (2, 0),
            (3, 0),
            (99, 0),
            (u32::MAX, 0),
        ] {
            let mut data = vec![0; shader.user_data_dwords() as usize];
            for u in &shader.user_data {
                data[u.index as usize] = match u.root {
                    RootType::Constants => index,
                    RootType::Table => 4 * 32,
                    _ => 0,
                };
            }
            let result = host
                .dispatch(
                    &root,
                    &pso,
                    &Dispatch {
                        roots: vec![
                            (0, Root::Constants(data)),
                            (
                                1,
                                Root::View {
                                    srv: true,
                                    data: metadata.clone(),
                                },
                            ),
                        ],
                        views: &views,
                        samplers: &[],
                        groups: [1, 1, 1],
                    },
                )
                .unwrap();
            assert_eq!(
                result.last().unwrap()[0],
                expected,
                "{backend}, heap slot {index}"
            );
        }
        eprintln!("{backend}: checked buffer, CBV, texture, UAV and out-of-range scalar heap loads");
    }
}
