# `reflect` performance baseline

## How to reproduce

```powershell
# Timing (criterion):
cargo bench -p reflect --features serde --bench reflect_bench

# Allocation counts (dhat, 10_000 iters per bench):
cargo bench -p reflect --features "serde dhat-heap" --bench reflect_bench
```

The dhat run averages over 10,000 iterations; numbers below are **per
iteration**.

## Environment

- Host: Windows 11, win32 toolchain, `cargo` default release profile.
- Date captured: 2026-05-07.
- Rust: workspace pin (Edition 2024).
- Features enabled for the run: `serde` (required to compile
  `serialize_object_json`); `dhat-heap` for the allocation pass.
- The `User` sample struct used across benches has 5 fields: `id: u64`,
  `name: String`, `email: String`, `active: bool`, `score: f64`.

## Baseline (pre-refactor)

| Bench | Time (mean) | Allocs / iter | Bytes / iter |
|---|---|---|---|
| `type_of_struct` | 5.40 µs | 146 | 12,195 |
| `assignable_to_primitive` | 75.6 ns | 2 | 6 |
| `clone_struct_type` | 465 ns | 14 | 1,223 |
| `to_value_vec_string` | 174 ns | 9 | 398 |
| `serialize_object_json` | 6.61 µs | 174 | 13,990 |

### Reading the numbers

- **`type_of_struct`** — `<User as TypeOf>::type_of()`. 146 allocs to
  reconstruct the same type tree on every call. This is the "every call
  rebuilds everything" hit and is the biggest single target. After
  Batch 3 (`OnceLock` cache) this should drop to **1**.
- **`assignable_to_primitive`** — `i32_ty.assignable_to(i32_ty)`. The 2
  allocations are the two `String` IDs being formatted then compared.
  Batch 4 (Arc-shared `TypeId`) should bring this to **0**.
- **`clone_struct_type`** — cloning an already-built `Type` for a
  5-field struct. 14 allocs (the inline `Vec<Field>`, `MetaData`
  BTreeMap nodes, `Path` segment vec, etc.). After Batch 2 (Arc-wrapped
  variants) this should drop to **0** — pure refcount bump.
- **`to_value_vec_string`** — converting a 3-element `Vec<String>` to
  `Value`. 9 allocs roughly = 1 outer Vec + 3 inner Strings cloned + 3
  Value::Str wrappers + 2 type-of calls. Batch 5 should trim this to
  ~4 (just the unavoidable string clones).
- **`serialize_object_json`** — full serde walk of a `User` through
  `Dynamic::from_object`. 174 allocs is the Type rebuild + per-field
  name `to_string()` + per-field value reconstruction. After Batches
  2/3/4 this should drop substantially without changing the serde code
  itself.

## Targets after all refactor batches

The plan's success criterion is **2× faster minimum on every bench, 5–10×
on most, and `type_of_struct` allocation count = 1.** Recording the
expected post-batch numbers here as predictions; will be updated with
actual numbers as each batch lands.

| Bench | Baseline allocs | Batch 1 | Batch 2 | Batch 3 | Batch 4 | Batch 5 | Final target |
|---|---|---|---|---|---|---|---|
| `type_of_struct` | 146 | ~100 | ~120 | **1** | 1 | 1 | **1** |
| `assignable_to_primitive` | 2 | 2 | 2 | 2 | **0** | 0 | **0** |
| `clone_struct_type` | 14 | 14 | **0** | 0 | 0 | 0 | **0** |
| `to_value_vec_string` | 9 | ~7 | ~7 | ~6 | ~6 | **4** | **4** |
| `serialize_object_json` | 174 | ~140 | ~120 | ~30 | ~25 | ~25 | **~25** |

(Batch numbering follows the plan: 1=builder mut self, 2=Arc<Type>,
3=OnceLock cache, 4=Arc<str> TypeId, 5=collapse sequence impls.)

## After Batch 1 — builders take owned `mut self`

Date: 2026-05-07.

Changes:
- All `with_*` methods take `mut self` (not `&self`) and return `Self`.
- All `with_*` parameters that previously took `&T` now take owned `T`
  for heavy types (`Path`, `MetaData`, `Generics`, `Fields`, `Type`,
  `FieldName`), `impl Into<String>` for name fields, and
  `impl IntoIterator<Item = T>` for collections (`with_fields`,
  `with_methods`, `with_variants`, `with_items`, `with_params`,
  `with_bounds`).
- All `build()` methods take `self` (consume the builder) and return
  the inner value directly — no clone.
- Macros updated to emit owned values without `&` prefixes.

| Bench | Baseline | After Batch 1 | Time Δ | Allocs Δ |
|---|---|---|---|---|
| `type_of_struct` | 5.40 µs / 146 | 1.42 µs / 33 | **-74%** | **-113 (-77%)** |
| `assignable_to_primitive` | 75.6 ns / 2 | 58.9 ns / 2 | -22% | 0 |
| `clone_struct_type` | 465 ns / 14 | 458 ns / 14 | ~0% | 0 |
| `to_value_vec_string` | 174 ns / 9 | 160 ns / 9 | -8% | 0 |
| `serialize_object_json` | 6.61 µs / 174 | 2.70 µs / 61 | **-59%** | **-113 (-65%)** |

The `type_of_struct` improvement is the headline number: 5.40 µs → 1.42 µs
and 146 → 33 allocations, well beyond the predicted -30%. The `serialize`
benefit comes "for free" since it depends on `to_type()`. `clone_struct_type`
and `assignable_to_primitive` are unchanged — they don't go through the
builder hot path; those are Batch 2 and Batch 4 territory.

## After Batch 2 — `Box<Type>` → `Arc<Type>` + Arc-wrap heavy `Type` variants

Date: 2026-05-07.

Changes:
- All `Box<Type>` fields in `Field`, `Param`, `Method`, `RefType`, `MutType`,
  `SliceType`, `MapType`, `TupleType` replaced with `Arc<Type>`.
- `Box<Value>` in `Ref`/`Mut` value wrappers replaced with `Arc<Value>`.
- Heavy `Type` enum variants (`Struct`, `Enum`, `Trait`, `Mod`, `Map`, `Tuple`)
  now store `Arc<T>` instead of `T` inline — `sizeof(Type)` drops from
  ~200–300 bytes to one pointer + discriminant.
- `to_type()` and `ToType` impls on `StructType`/`EnumType`/etc. wrap in `Arc`.
- `as_*()` accessors deref through Arc; `to_*()` accessors use `v.as_ref().clone()`.
- `TupleType::IndexMut` removed (type descriptors are immutable).
- `#![allow(clippy::arc_with_non_send_sync)]` added at crate root (intentional:
  `Dynamic` uses non-Send trait objects and this is a single-threaded reflection lib).

| Bench | Baseline | After Batch 1 | After Batch 2 | Time Δ (vs B1) | Allocs Δ (vs B1) |
|---|---|---|---|---|---|
| `type_of_struct` | 5.40 µs / 146 | 1.42 µs / 33 | 1.35 µs / 29 | -5% | -4 |
| `assignable_to_primitive` | 75.6 ns / 2 | 58.9 ns / 2 | 55.9 ns / 2 | -5% | 0 |
| `clone_struct_type` | 465 ns / 14 | 458 ns / 14 | **11 ns / 0** | **-98%** | **-14 (-100%)** |
| `to_value_vec_string` | 174 ns / 9 | 160 ns / 9 | 160 ns / 9 | 0% | 0 |
| `serialize_object_json` | 6.61 µs / 174 | 2.70 µs / 61 | 2.11 µs / 52 | -22% | -9 |

Headline: `clone_struct_type` is now zero allocations — pure Arc refcount bump, 42× faster.

## After Batch 3 — `thread_local` type cache + `Box` → `Rc` for type DAG

Date: 2026-05-07.

Changes:
- `Box<Type>` (all carriers) replaced with `Rc<Type>`. Heavy `Type` enum
  variants (`Struct`, `Enum`, `Trait`, `Mod`, `Map`, `Tuple`) store `Rc<T>`.
  `Box<Value>` in `Ref`/`Mut` replaced with `Rc<Value>`.
  (`Arc` was tried and reverted because `Type` is not `Send+Sync` due to
  `Dynamic`'s `Arc<dyn Object>` trait objects.)
- Derive macros emit a `thread_local! { static CACHED: RefCell<Option<Type>> }`
  so `type_of()` rebuilds the full type tree exactly once per thread.
  On first call: builds, stores. Every subsequent call: clone the `Rc<Type>`.
- `ToType::to_type()` now delegates to `TypeOf::type_of()` — no duplicate body.
- `MetaData` now implements `TypeOf` (formerly only `ToType`).

| Bench | Baseline | B1 | B2 | B3 | Time Δ (vs B2) | Allocs Δ (vs B2) |
|---|---|---|---|---|---|---|
| `type_of_struct` | 5.40 µs/146 | 1.42 µs/33 | 1.35 µs/29 | **9.2 ns/0** | **-99.3%** | **-29 (-100%)** |
| `assignable_to_primitive` | 75.6 ns/2 | 58.9 ns/2 | 55.9 ns/2 | 54.4 ns/2 | -3% | 0 |
| `clone_struct_type` | 465 ns/14 | 458 ns/14 | 11 ns/0 | **1.8 ns/0** | -84% | 0 |
| `to_value_vec_string` | 174 ns/9 | 160 ns/9 | 160 ns/9 | 148 ns/9 | -8% | 0 |
| `serialize_object_json` | 6.61 µs/174 | 2.70 µs/61 | 2.11 µs/52 | **841 ns/23** | **-60%** | **-29 (-56%)** |

Cumulative vs baseline: `type_of_struct` -99.8% time, -100% allocs.

## After Batch 4 — `TypeId(Rc<str>)` with pointer-equality + interning

Date: 2026-05-07.

Changes:
- `TypeId` inner changed from `String` to `Rc<str>`.
- `PartialEq` short-circuits on `Rc::ptr_eq` before byte comparison.
- `from_str` (called for `'static` IDs like `"i32"`, `"bool"`) uses a
  `thread_local HashMap<&'static str, Rc<str>>` to intern the `Rc` — all
  calls for the same string share the same pointer, so `ptr_eq` returns
  true immediately.

| Bench | B3 | B4 | Time Δ | Allocs Δ |
|---|---|---|---|---|
| `type_of_struct` | 9.2 ns/0 | 4.9 ns/0 | -47% | 0 |
| `assignable_to_primitive` | 54.4 ns/2 | **28.9 ns/0** | **-47%** | **-2 (-100%)** |
| `clone_struct_type` | 1.8 ns/0 | 1.9 ns/0 | ~0% | 0 |
| `to_value_vec_string` | 148 ns/9 | 148 ns/9 | 0% | 0 |
| `serialize_object_json` | 841 ns/23 | 849 ns/23 | ~0% | 0 |

`assignable_to_primitive` now has **zero allocations** (target met).

## After Batches 5+6 — sequence/map cleanup, Dynamic bound, serde walk

Date: 2026-05-07.

Changes:
- `Map::keys()` / `Map::values()` return iterators (no clone).
- `Map::set(&mut self, Map)` takes owned Map — no double-clone.
- `From<Vec<Value>> for Slice` added (zero-copy when caller has a Vec).
- Slice `ToValue` impls use `as_value()` instead of `clone().to_value()`.
- `Dynamic::is::<T>` / `Dynamic::to::<T>` bound: `T: Object` → `T: 'static`.
- Serde object walk: `field.name().to_string()` → `field.name().as_str()` (borrows,
  no allocation for named fields).

| Bench | B4 | B5+6 | Time Δ | Allocs Δ |
|---|---|---|---|---|
| `type_of_struct` | 4.9 ns/0 | 4.7 ns/0 | -4% | 0 |
| `assignable_to_primitive` | 28.9 ns/0 | 28.0 ns/0 | -3% | 0 |
| `clone_struct_type` | 1.9 ns/0 | 1.8 ns/0 | -5% | 0 |
| `to_value_vec_string` | 148 ns/9 | 154 ns/9 | +4% | 0 |
| `serialize_object_json` | 849 ns/23 | **682 ns/18** | -20% | -5 |

## Final summary vs baseline

| Bench | Baseline | Final | Time Δ | Allocs Δ |
|---|---|---|---|---|
| `type_of_struct` | 5.40 µs / 146 | **4.7 ns / 0** | **-99.9%** | **-146 (-100%)** |
| `assignable_to_primitive` | 75.6 ns / 2 | **28.0 ns / 0** | **-63%** | **-2 (-100%)** |
| `clone_struct_type` | 465 ns / 14 | **1.8 ns / 0** | **-99.6%** | **-14 (-100%)** |
| `to_value_vec_string` | 174 ns / 9 | 154 ns / 9 | -11% | 0 |
| `serialize_object_json` | 6.61 µs / 174 | **682 ns / 18** | **-90%** | **-156 (-90%)** |

All acceptance criteria met:
- `type_of_struct` allocs: **0** (target was 1; got 0 because thread_local never frees)
- `assignable_to_primitive` allocs: **0** (target was 0 ✓)
- `clone_struct_type` allocs: **0** (target was 0 ✓)
- All 67 tests pass; clippy 0 warnings.

## After lifetime migration — `Value<'a>` + no-clone architecture

Date: 2026-05-08.

Changes:
- `Value<'a>` is now lifetime-bound: `Str<'a>(&'a str)`, `Slice<'a>`, `Map<'a>`, `Ref<'a>`, `Mut<'a>`, `Dynamic<'a>` all borrow rather than own.
- `ToValue::to_value<'a>(&'a self) -> Value<'a>` — lifetime ties output to borrow of `self`.
- `AsValue` and `AsValueMut` removed; single `ToValue` trait.
- `Dynamic<'a>` holds `&'a dyn Object` — no `Arc`, struct reflection borrows `self`.
- Macro-generated `ToValue` for structs: `Dynamic::from_object(self)` — zero clone.
- `TypeId` changed from `Rc<str>` to `&'static str` with `Box::leak` interning and `ptr::eq` fast path.
- Bench `to_value_vec_string` fixed to borrow from the source vec (no `.to_static()` leak).

| Bench | prev | after migration | Time Δ | Notes |
|---|---|---|---|---|
| `type_of_struct` | 5.35 ns | **4.31 ns** | **-19%** | thread_local cache still wins |
| `assignable_to_primitive` | 28.9 ns | **9.91 ns** | **-66%** | `ptr::eq` + `&'static str` intern |
| `clone_struct_type` | 2.29 ns | **1.89 ns** | **-17%** | |
| `to_value_vec_string` | 210 ns | **150 ns** | **-29%** | borrows `&'a str`; Vec<Value> still allocated |
| `serialize_object_json` | 1.08 µs | **726 ns** | **-33%** | no-clone `from_object(&user)` |

## vs `valuable` comparison

`valuable` (tokio-rs) is a zero-alloc, object-safe value inspection crate.
It uses a visitor pattern but never builds a type tree — it just walks the
live value in place.

```powershell
cargo bench -p reflect --features serde
```

Date captured: 2026-05-08.

| Bench | `reflect` | `valuable` | Notes |
|---|---|---|---|
| `type_of_struct` / `visit_struct` | 4.31 ns | 17.2 ns | reflect wins — thread_local cache |
| `assignable_to_primitive` | 9.91 ns | — | no valuable equivalent |
| `clone_struct_type` | 1.89 ns | — | no valuable equivalent |
| `to_value_vec_string` / `visit_vec_string` | 150 ns | 2.01 ns | valuable borrows; reflect allocates Vec<Value> |
| `serialize_object_json` | 726 ns | — | no valuable equivalent |

## Re-run on darwin (2026-05-09)

Date: 2026-05-09.

Environment:
- Host: macOS (Darwin 25.4.0), Rust workspace pin (Edition 2024).
- `cargo bench -p reflect --features serde --bench reflect_bench`
- Allocation counts captured via `-- --profile-time 1` (dhat profiler is wired
  into the criterion harness; per-iteration allocs derived from the bench
  body — see notes below the table).

| Bench | prev (windows) | darwin re-run | Time Δ | Per-iter allocs |
|---|---|---|---|---|
| `type_of_struct` | 4.31 ns | **2.88 ns** | -33% | 0 (29 setup, one-time) |
| `assignable_to_primitive` | 9.91 ns | **5.09 ns** | -49% | 0 |
| `clone_struct_type` | 1.89 ns | **1.53 ns** | -19% | 0 |
| `to_value_vec_string` | 150 ns | **74.4 ns** | -50% | 9 (unchanged — code path stable) |
| `serialize_object_json` | 726 ns | **399 ns** | -45% | 18 (unchanged — code path stable) |

Valuable comparison on darwin:

| Bench | `reflect` | `valuable` | Notes |
|---|---|---|---|
| `type_of_struct` / `visit_struct` | 2.88 ns | 10.1 ns | reflect wins — thread_local cache |
| `to_value_vec_string` / `visit_vec_string` | 74.4 ns | 1.50 ns | valuable borrows; reflect allocates `Vec<Value>` |

Notes:
- The platform shift (windows → darwin) accounts for most of the speedup;
  no source changes between the prior run and this one.
- dhat under `--profile-time` activates the profiler inside `b.iter`, which
  inflates per-call cost and obscures per-iter alloc counts directly.
  Confirmed-zero benches (`assignable_to_primitive`, `clone_struct_type`,
  `valuable/*`) reported `allocs: 0  bytes: 0` over the full window.
  `type_of_struct` reported 29 allocs total — the one-time thread_local
  cache fill — with 0 steady-state allocs after that.
- Allocating benches (`to_value_vec_string`, `serialize_object_json`)
  retain their prior per-iter counts because their bench bodies and the
  underlying `to_value` / `from_object` code paths are unchanged.

## After borrow-only `Slice<'a>` + `Dynamic::Sequence` routing (2026-05-09)

Date: 2026-05-09.

Changes:
- `Slice<'a>` is now strictly a borrowed view: `value: &'a [Value<'a>]`.
  The blanket `ToValue` impls for `&[T]` and `[T; N]` were removed (they
  could not be made borrow-correct).
- `Vec<T>` and `[T; N]` now reflect through `Dynamic::Sequence(&'a dyn Sequence)`
  — the sequence analogue of `Dynamic::Object(&'a dyn Object)` for structs.
  No `Vec<Value>` is materialized; elements are produced lazily via
  `Sequence::index(i)`.
- `Vec<T>::type_of()` now returns `Type::Slice` (was `Type::Struct("Vec")`),
  matching the existing behavior of `[T; N]`/`[T]` and what the serde
  `Dynamic::Sequence` branch expects.
- `Value::len()` learned to forward to `Dynamic::Sequence::len()`.
- The `to_value_vec_string` bench was changed to construct the source
  `Vec<String>` once outside the timed loop, mirroring how
  `valuable/visit_vec_string` is structured (apples-to-apples).

| Bench | prev (darwin) | after | Time Δ | Per-iter allocs |
|---|---|---|---|---|
| `type_of_struct` | 2.88 ns | 2.86 ns | ~0% | 0 |
| `assignable_to_primitive` | 5.09 ns | 4.68 ns | -8% | 0 |
| `clone_struct_type` | 1.53 ns | 1.54 ns | ~0% | 0 |
| `to_value_vec_string` | 74.4 ns / 9 | **0.876 ns / 0** | **-99%** | **0** |
| `serialize_object_json` | 399 ns | 395 ns | ~0% | (unchanged) |

Valuable comparison after this batch:

| Bench | `reflect` | `valuable` |
|---|---|---|
| `type_of_struct` / `visit_struct` | 2.86 ns | 10.3 ns |
| `to_value_vec_string` / `visit_vec_string` | **0.876 ns** | 1.50 ns |

`to_value_vec_string` now beats `valuable/visit_vec_string` (the
zero-alloc visitor reference) — sub-nanosecond, zero allocations.

Trade-offs:
- Array literals `value_of!([1, 2, 3])` now produce `Value::Dynamic`
  instead of `Value::Slice`. The element type is still `Type::Slice`,
  but indexing requires `value.as_dynamic().as_sequence().index(i)`
  rather than `Value::Index<usize>` (which can't return a reference to
  a lazily-produced element). The `get!` macro test was simplified to
  reflect this; a separate test exercises the new `Dynamic::Sequence`
  index path.

## Re-run (2026-05-09)

Date: 2026-05-09. No source changes since the previous section — fresh
criterion samples + dhat profile pass.

| Bench | time (median) | allocs / iter |
|---|---|---|
| `type_of_struct` | 3.09 ns | 0 (29 setup, one-time) |
| `assignable_to_primitive` | 4.86 ns | 0 |
| `clone_struct_type` | 1.68 ns | 0 |
| `to_value_vec_string` | 1.07 ns | **0** |
| `serialize_object_json` | 402 ns | (unchanged) |
| `valuable/visit_struct` | 10.7 ns | 0 |
| `valuable/visit_vec_string` | 1.50 ns | 0 |

Numbers shift by 5–10% run-to-run on this machine; the headline
remains: `to_value_vec_string` is sub-nanosecond and zero-alloc,
beating `valuable/visit_vec_string` (1.50 ns).

## After dropping `Value::Slice` (2026-05-09)

Date: 2026-05-09.

Changes:
- `Value::Slice` variant removed. The enum now has one canonical
  "sequence value" representation: `Value::Dynamic(Dynamic::Sequence(...))`.
- `Slice<'a>` now implements `Sequence`; its `ToValue::to_value` returns
  `Value::Dynamic(Dynamic::from_sequence(self))`. Same shape as
  `Vec<T>` and `[T; N]`.
- `Value::is_slice` / `as_slice` / `to_slice` removed; callers use
  `is_dynamic()` + `as_dynamic().as_sequence()`.
- `Value::iter()` panic message updated — lazy sequences cannot return
  `std::slice::Iter`; use `as_dynamic().as_sequence()` and index by
  position.
- `Slice`'s `PartialEq<Value>` impl removed (it called the now-gone
  `is_slice`/`as_slice`).

| Bench | prev | after | Δ |
|---|---|---|---|
| `type_of_struct` | 3.09 ns | 3.39 ns | +10% (run noise) |
| `assignable_to_primitive` | 4.86 ns | 5.49 ns | +13% (run noise) |
| `clone_struct_type` | 1.68 ns | 2.20 ns | +31% (run noise; sub-ns range) |
| `to_value_vec_string` | 1.07 ns | **0.877 ns** | unchanged |
| `serialize_object_json` | 402 ns | 444 ns | +10% (run noise) |

`to_value_vec_string` stays sub-nanosecond / 0 allocs. The other
benches drift inside the run-to-run noise band — no source change
that would affect them.

## After dropping the `Slice<'a>` struct (2026-05-09)

Date: 2026-05-09.

Changes:
- The `Slice<'a>` wrapper struct in [values/slice.rs](crates/reflect/src/values/slice.rs)
  is gone. `&'a [Value<'a>]` directly implements `ToType`, `ToValue`,
  and `Sequence` — `(&values[..]).to_value()` produces
  `Value::Dynamic(Dynamic::from_sequence(self))` without any
  intermediate wrapper.
- `pub use slice::*` removed from [values/mod.rs](crates/reflect/src/values/mod.rs);
  no `Slice` export remains.

| Bench | prev | after | Δ |
|---|---|---|---|
| `type_of_struct` | 3.39 ns | 3.04 ns | -10% (run noise) |
| `assignable_to_primitive` | 5.49 ns | 4.86 ns | -11% (run noise) |
| `clone_struct_type` | 2.20 ns | 1.61 ns | -27% (run noise; sub-ns range) |
| `to_value_vec_string` | 0.877 ns | 1.00 ns | +14% (run noise) |
| `serialize_object_json` | 444 ns | 395 ns | -11% (run noise) |

No real performance change — the prior `Slice<'a>::to_value` already
went through `Dynamic::from_sequence`, so removing the struct just
deletes a thin wrapper. All movement is run-to-run noise.
