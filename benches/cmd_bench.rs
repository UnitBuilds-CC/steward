use criterion::{black_box, criterion_group, criterion_main, Criterion};
use steward::{Cmd, Env, KillTimeout, Location};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone)]
struct BenchLoc(PathBuf);

impl BenchLoc {
    fn root() -> Self {
        Self(std::env::current_dir().unwrap())
    }
}

impl Location for BenchLoc {
    fn apex() -> Self { Self::root() }
    fn as_path(&self) -> &PathBuf { &self.0 }
    fn join<P: AsRef<Path>>(&self, path: P) -> Self { Self(self.0.join(path)) }
}

fn kill_timeout(c: &mut Criterion) {
    let mut group = c.benchmark_group("kill_timeout");

    group.bench_function("default", |b| {
        b.iter(|| KillTimeout::default())
    });

    group.bench_function("new", |b| {
        b.iter(|| KillTimeout::new(black_box(Duration::from_secs(10))))
    });

    group.bench_function("from_duration", |b| {
        b.iter(|| {
            let _: KillTimeout = black_box(Duration::from_secs(10)).into();
        })
    });

    group.bench_function("duration_access", |b| {
        let timeout = KillTimeout::new(Duration::from_secs(10));
        b.iter(|| black_box(timeout.duration()))
    });

    group.finish();
}

fn shelled(c: &mut Criterion) {
    c.bench_function("shelled_array", |b| {
        b.iter(|| {
            // This benchmarks the stack-allocated [&str; 2] approach
            let cmd = black_box("echo hello world");
            let result: [&str; 2] = if cfg!(unix) {
                ["-c", cmd]
            } else {
                ["/c", cmd]
            };
            black_box(result)
        })
    });
}

fn headline(c: &mut Criterion) {
    let cmd_with_msg = Cmd {
        exe: "cargo build --release".to_string(),
        env: Env::empty(),
        pwd: BenchLoc::root(),
        msg: Some("Building project".to_string()),
    };

    let cmd_without_msg = Cmd {
        exe: "cargo build --release".to_string(),
        env: Env::empty(),
        pwd: BenchLoc::root(),
        msg: None,
    };

    c.bench_function("headline_with_msg", |b| {
        b.iter(|| {
            steward::headline!(black_box(&cmd_with_msg))
        })
    });

    c.bench_function("headline_without_msg", |b| {
        b.iter(|| {
            steward::headline!(black_box(&cmd_without_msg))
        })
    });
}

fn cmd_construction(c: &mut Criterion) {
    c.bench_function("cmd_struct_construction", |b| {
        b.iter(|| {
            Cmd {
                exe: black_box("cargo build").to_string(),
                env: Env::empty(),
                pwd: BenchLoc::root(),
                msg: Some(black_box("Building").to_string()),
            }
        })
    });

    c.bench_function("cmd_macro_construction", |b| {
        b.iter(|| {
            steward::cmd! {
                "cargo build",
                env: Env::empty(),
                pwd: BenchLoc::root(),
                msg: "Building",
            }
        })
    });
}

fn spawn_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_overhead");
    group.sample_size(10);

    let cmd = Cmd {
        exe: if cfg!(windows) {
            "echo bench".to_string()
        } else {
            "echo bench".to_string()
        },
        env: Env::empty(),
        pwd: BenchLoc::root(),
        msg: None,
    };

    group.bench_function("spawn_quick_command", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                let opts = steward::SpawnOptions {
                    stdout: std::process::Stdio::null(),
                    stderr: std::process::Stdio::null(),
                    ..Default::default()
                };
                let running = cmd.spawn(opts).unwrap();
                let _ = running.into_child();
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    kill_timeout,
    shelled,
    headline,
    cmd_construction,
    spawn_overhead,
);
criterion_main!(benches);
