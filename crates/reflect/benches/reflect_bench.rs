//! Performance baseline for reflect vs valuable.
//!
//! Run with:
//!   cargo bench -p reflect --features serde
//!
//! Each benchmark emits a criterion timing line plus a dhat allocation delta
//! on stderr, e.g.:
//!   dhat [type_of_struct]  allocs: 0  bytes: 0

#![allow(unused)]

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use std::path::Path;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use reflect::{ToType, ToValue, TypeOf};
use reflect_macros::Reflect;
use valuable::Valuable;

// ----- dhat criterion profiler -----

struct DhatProfiler {
    profiler: Option<dhat::Profiler>,
    before: Option<dhat::HeapStats>,
}

impl DhatProfiler {
    fn new() -> Self {
        Self {
            profiler: None,
            before: None,
        }
    }
}

impl criterion::profiler::Profiler for DhatProfiler {
    fn start_profiling(&mut self, _id: &str, _dir: &Path) {
        self.profiler = Some(dhat::Profiler::builder().testing().build());
        self.before = Some(dhat::HeapStats::get());
    }

    fn stop_profiling(&mut self, id: &str, _dir: &Path) {
        let after = dhat::HeapStats::get();
        if let Some(before) = self.before.take() {
            let allocs = after.total_blocks - before.total_blocks;
            let bytes = after.total_bytes - before.total_bytes;
            eprintln!("  dhat [{id}]  allocs: {allocs}  bytes: {bytes}");
        }
        self.profiler = None;
    }
}

fn custom_criterion() -> Criterion {
    Criterion::default().with_profiler(DhatProfiler::new())
}

// ----- shared sample types -----

#[derive(Debug, Clone, Reflect, Valuable)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub active: bool,
    pub score: f64,
}

fn sample_user() -> User {
    User {
        id: 42,
        name: "alex".to_string(),
        email: "alex@example.com".to_string(),
        active: true,
        score: 99.5,
    }
}

fn sample_strings() -> Vec<String> {
    vec!["a".to_string(), "b".to_string(), "c".to_string()]
}

// ----- reflect bench bodies -----

#[inline(never)]
fn bench_type_of_struct() -> reflect::Type {
    <User as TypeOf>::type_of()
}

#[inline(never)]
fn bench_assignable_to_primitive() -> bool {
    let lhs = i32::type_of();
    let rhs = i32::type_of();
    lhs.assignable_to(rhs)
}

#[inline(never)]
fn bench_clone_struct_type(t: &reflect::Type) -> reflect::Type {
    t.clone()
}

#[inline(never)]
fn bench_to_value_vec_string(v: &Vec<String>) {
    let _ = std::hint::black_box(v.to_value());
}

#[inline(never)]
fn bench_serialize_object_json(user: &User) -> String {
    let dynamic = reflect::Dynamic::from_object(user);
    serde_json::to_string(&dynamic).expect("serialize")
}

// ----- valuable bench bodies -----

struct NoopVisitor;

impl valuable::Visit for NoopVisitor {
    fn visit_value(&mut self, value: valuable::Value<'_>) {
        match value {
            valuable::Value::Structable(v) => v.visit(self),
            valuable::Value::Listable(v) => v.visit(self),
            _ => {}
        }
    }

    fn visit_named_fields(&mut self, named_fields: &valuable::NamedValues<'_>) {
        for (_, v) in named_fields.iter() {
            v.visit(self);
        }
    }

    fn visit_primitive_slice(&mut self, _: valuable::Slice<'_>) {}
}

#[inline(never)]
fn bench_valuable_visit_struct(user: &User) {
    user.visit(&mut NoopVisitor);
}

#[inline(never)]
fn bench_valuable_visit_vec_string(v: &Vec<String>) {
    v.visit(&mut NoopVisitor);
}

// ----- reflect criterion fns -----

fn type_of_struct(c: &mut Criterion) {
    c.bench_function("type_of_struct", |b| {
        b.iter(|| black_box(bench_type_of_struct()));
    });
}

fn assignable_to_primitive(c: &mut Criterion) {
    c.bench_function("assignable_to_primitive", |b| {
        b.iter(|| black_box(bench_assignable_to_primitive()));
    });
}

fn clone_struct_type(c: &mut Criterion) {
    let t = <User as TypeOf>::type_of();
    c.bench_function("clone_struct_type", |b| {
        b.iter(|| black_box(bench_clone_struct_type(&t)));
    });
}

fn to_value_vec_string(c: &mut Criterion) {
    let v = sample_strings();
    c.bench_function("to_value_vec_string", |b| {
        b.iter(|| bench_to_value_vec_string(&v));
    });
}

fn serialize_object_json(c: &mut Criterion) {
    let user = sample_user();
    c.bench_function("serialize_object_json", |b| {
        b.iter(|| black_box(bench_serialize_object_json(&user)));
    });
}

// ----- valuable criterion fns -----

fn valuable_visit_struct(c: &mut Criterion) {
    let user = sample_user();
    c.bench_function("valuable/visit_struct", |b| {
        b.iter(|| black_box(bench_valuable_visit_struct(&user)));
    });
}

fn valuable_visit_vec_string(c: &mut Criterion) {
    let v = sample_strings();
    c.bench_function("valuable/visit_vec_string", |b| {
        b.iter(|| black_box(bench_valuable_visit_vec_string(&v)));
    });
}

// ----- groups -----

criterion_group! {
    name = benches;
    config = custom_criterion();
    targets = type_of_struct, assignable_to_primitive, clone_struct_type,
              to_value_vec_string, serialize_object_json
}

criterion_group! {
    name = valuable_benches;
    config = custom_criterion();
    targets = valuable_visit_struct, valuable_visit_vec_string
}

criterion_main!(benches, valuable_benches);
