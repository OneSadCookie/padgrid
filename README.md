At [4172b22](https://github.com/OneSadCookie/padgrid/commit/4172b22aca8d5ac43d5d99d0455a6f5c046a16a4),
demonstrated a crash with `impl trait`. Works on latest compiler versions.

```
error: internal compiler error: src/librustc_trans/common.rs:473: Encountered er
ror `Unimplemented` selecting `Binder(<std::vec::Vec<futures::future::join_all::
ElemState<futures::OrElse<futures::AndThen<futures::FutureResult<usize, std::num
::ParseIntError>, futures::OrElse<futures::Map<futures::AndThen<futures::OrElse<
futures::FutureResult<std::fs::File, PadGridError>, futures::OrElse<futures::And
Then<futures::AndThen<futures::AndThen<futures::future::FromErr<hyper::client::F
utureResponse, PadGridError>, std::result::Result<hyper::client::Response, PadGr
idError>, [closure@src/main.rs:79:41: 85:6 url_string:std::string::String]>, fut
ures::AndThen<futures::AndThen<futures::future::FromErr<futures::FutureResult<st
d::fs::File, std::io::Error>, PadGridError>, futures::future::FromErr<futures::s
tream::Fold<hyper::Body, [closure@src/main.rs:62:32: 67:10], std::result::Result
<std::fs::File, std::io::Error>, std::fs::File>, PadGridError>, [closure@src/mai
n.rs:61:86: 68:6 res:hyper::client::Response]>, std::result::Result<(), PadGridE
rror>, [closure@src/main.rs:68:17: 68:28]>, [closure@src/main.rs:85:17: 87:6 id:
usize]>, futures::FutureResult<std::fs::File, PadGridError>, [closure@src/main.r
s:87:17: 90:6 id:usize]>, std::result::Result<std::fs::File, PadGridError>, [clo
sure@src/main.rs:90:16: 94:6 id:usize]>, [closure@src/main.rs:101:33: 103:6 id:u
size, handle:tokio_core::reactor::Handle]>, std::result::Result<image::DynamicIm
age, PadGridError>, [closure@src/main.rs:110:44: 112:6]>, [closure@src/main.rs:1
50:38: 152:10]>, std::result::Result<GridCell, std::num::ParseIntError>, [closur
e@src/main.rs:152:20: 154:10]>, [closure@src/main.rs:149:57: 155:6 handle:tokio_
core::reactor::Handle]>, std::result::Result<GridCell, PadGridError>, [closure@s
rc/main.rs:155:16: 157:6 fallback_cell:GridCell]>>> as std::iter::IntoIterator>)
` during trans

note: the compiler unexpectedly panicked. this is a bug.

note: we would appreciate a bug report: https://github.com/rust-lang/rust/blob/master/CONTRIBUTING.md#bug-reports

note: run with `RUST_BACKTRACE=1` for a backtrace

thread 'rustc' panicked at 'Box<Any>', src/librustc_errors/lib.rs:376
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.
stack backtrace:
   0: rustc::session::opt_span_bug_fmt::{{closure}}
   1: rustc::session::span_bug_fmt
   2: rustc_trans::common::fulfill_obligation::{{closure}}::{{closure}}
   3: rustc_trans::common::fulfill_obligation
   4: rustc_trans::collector::do_static_dispatch
   5: <rustc_trans::collector::MirNeighborCollector<'a, 'tcx> as rustc::mir::visit::Visitor<'tcx>>::visit_operand
   6: <rustc_trans::collector::MirNeighborCollector<'a, 'tcx> as rustc::mir::visit::Visitor<'tcx>>::visit_terminator_kind
   7: rustc::mir::visit::Visitor::visit_mir
   8: rustc_trans::collector::collect_neighbours
   9: rustc_trans::collector::collect_items_rec
  10: rustc_trans::collector::collect_items_rec
  11: rustc_trans::collector::collect_items_rec
  12: rustc_trans::collector::collect_items_rec
  13: rustc_trans::collector::collect_items_rec
  14: rustc_trans::collector::collect_items_rec
  15: rustc_trans::collector::collect_items_rec
  16: rustc_trans::collector::collect_items_rec
  17: rustc_trans::collector::collect_items_rec
  18: rustc_trans::collector::collect_items_rec
  19: rustc_trans::collector::collect_items_rec
  20: rustc_trans::collector::collect_items_rec
  21: rustc_trans::collector::collect_items_rec
  22: rustc_trans::collector::collect_items_rec
  23: rustc_trans::base::collect_and_partition_translation_items::{{closure}}
  24: rustc_trans::base::collect_and_partition_translation_items
  25: rustc_trans::base::trans_crate
  26: rustc_driver::driver::phase_4_translate_to_llvm
  27: rustc_driver::driver::compile_input::{{closure}}
  28: rustc_driver::driver::phase_3_run_analysis_passes::{{closure}}
  29: rustc_driver::driver::phase_3_run_analysis_passes
  30: rustc_driver::driver::compile_input
  31: rustc_driver::run_compiler
  32: std::panicking::try::do_call
  33: __rust_maybe_catch_panic
  34: <F as alloc::boxed::FnBox<A>>::call_box
  35: std::sys::imp::thread::Thread::new::thread_start
  36: _pthread_body
  37: _pthread_start

error: Could not compile `padgrid`.
```