//! Console register packing must not mix host interpolation modes.

use super::bitcode::{Attr, Term};
use super::container::SigElement;
use super::shader::Shader;

#[test]
fn mixed_interpolation_outputs_validate_without_changing_semantics() {
    let element = |name: &str, index, comp, interp, row, col, cols| SigElement {
        name: name.into(),
        index,
        kind: 0,
        comp,
        interp,
        row: Some(row),
        col,
        cols,
        usage: (1 << cols) - 1,
        half: false,
    };
    let mut shader = Shader::new(false);
    let mut position = element("SV_Position", 0, 9, 4, 0, 0, 4);
    position.kind = 3;
    shader.outputs = vec![
        position,
        element("TEXCOORD", 0, 9, 2, 1, 0, 3),
        element("TEXCOORD", 1, 5, 1, 1, 3, 1),
        // Declaration order within a shared row differs from column order.
        element("COLOR", 1, 9, 2, 2, 3, 1),
        element("TEXCOORD", 2, 9, 2, 2, 0, 3),
    ];
    for id in 0..shader.outputs.len() {
        let e = shader.outputs[id].clone();
        for col in 0..e.cols {
            let (suffix, zero) = if e.comp == 9 {
                ("f32", shader.m.f32(0))
            } else {
                ("i32", shader.m.i32(0))
            };
            let args = [
                shader.m.i32(id as u32),
                shader.m.i32(0),
                shader.m.i8(col),
                zero,
            ];
            shader.op(
                shader.entry,
                &format!("storeOutput.{suffix}"),
                Attr::NoUnwind,
                shader.t.void,
                5,
                &args,
            );
        }
    }
    shader.m.terminate(shader.entry, Term::Ret);
    let mut bytes = shader.finish();
    if let Ok(mut validator) =
        super::validator::Validator::load(super::validator::Validator::default_path().as_deref())
    {
        validator.validate(&mut bytes).unwrap();
    }
    let container = crate::Container::parse(&bytes).unwrap();
    let outputs = super::signature(container.part(crate::FourCC::OSG1).unwrap().data).unwrap();
    assert_eq!(outputs.len(), 5);
    let float = outputs
        .iter()
        .find(|e| e.name == "TEXCOORD" && e.index == 0)
        .unwrap();
    let integer = outputs
        .iter()
        .find(|e| e.name == "TEXCOORD" && e.index == 1)
        .unwrap();
    assert_ne!(float.row, integer.row);
    assert_eq!((float.comp, float.col, float.cols), (9, 0, 3));
    assert_eq!((integer.comp, integer.col, integer.cols), (5, 3, 1));
}
