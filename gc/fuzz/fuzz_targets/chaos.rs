#![no_main]
use std::borrow::Borrow;

use libfuzzer_sys::fuzz_target;

use arbitrary::Arbitrary;
use gc::{Finalize, Gc, GcCell, Trace};

#[derive(Trace, Finalize)]
struct Foo(GcCell<Option<Gc<Foo>>>);

#[derive(Debug, Arbitrary)]
enum GcOp {
    New,
    NewBox(u8),
    NewCyclic,
    Collect,
    TryBorrow(u8),
    Finalize(u8),
    Drop(u8),
    DropBox(u8),
    DropCyclic(u8),
}

fuzz_target!(|ops: Vec<GcOp>| {
    let mut refs = Vec::new();
    let mut boxrefs = Vec::new();
    let mut cyclicrefs = Vec::new();
    for op in ops {
        match op {
            GcOp::New => refs.push(Gc::new(12u8)),
            GcOp::NewBox(i) => {
                let i = i as usize;
                if i < 128 && i < refs.len() {
                    boxrefs.push(Gc::new(Box::new(refs[i].clone())));
                } else {
                    boxrefs.push(Gc::new(Box::new(Gc::new(13u8))));
                }
            }
            GcOp::NewCyclic => {
                let f = Gc::new(Foo(GcCell::new(None)));
                let g = Gc::new(Foo(GcCell::new(Some(f.clone()))));
                *f.0.borrow_mut() = Some(g.clone());
                cyclicrefs.push(f);
            }
            GcOp::Collect => gc::force_collect(),
            GcOp::TryBorrow(i) => {
                let _: Option<&u8> = refs.get(i as usize).map(|r| r.borrow());
            }
            GcOp::Finalize(i) => {
                refs.get(i as usize).map(|r| r.finalize());
            }
            GcOp::Drop(i) => {
                let i = i as usize;
                if i < refs.len() {
                    refs.remove(i);
                }
            }
            GcOp::DropBox(i) => {
                let i = i as usize;
                if i < boxrefs.len() {
                    boxrefs.remove(i);
                }
            }
            GcOp::DropCyclic(i) => {
                let i = i as usize;
                if i < cyclicrefs.len() {
                    cyclicrefs.remove(i);
                }
            }
        }
    }
});
