use criterion::{Criterion, criterion_group, criterion_main};
use ginyu_poses::{Pose, pose};
use std::hint::black_box;

fn bench_static_pose_macro(c: &mut Criterion) {
    c.bench_function("static pose via macro", |bencher| {
        bencher.iter(|| black_box(pose!("color")));
    });
}

fn bench_static_pose_from(c: &mut Criterion) {
    c.bench_function("static pose via From", |bencher| {
        bencher.iter(|| black_box(Pose::from("color")));
    });
}

fn bench_dynamic_pose_first(c: &mut Criterion) {
    c.bench_function("dynamic pose (first intern)", |bencher| {
        let mut counter = 0u64;
        bencher.iter(|| {
            counter += 1;
            let s = format!("dynamic-prop-{counter}");
            black_box(Pose::from(s.as_str()))
        });
    });
}

fn bench_dynamic_pose_cached(c: &mut Criterion) {
    // Pre-intern this string
    let _ = Pose::from("cached-dynamic-prop");

    c.bench_function("dynamic pose (cached lookup)", |bencher| {
        bencher.iter(|| black_box(Pose::from("cached-dynamic-prop")));
    });
}

fn bench_pose_equality_static(c: &mut Criterion) {
    let a = pose!("color");
    let b = pose!("color");

    c.bench_function("equality (static)", |bencher| {
        bencher.iter(|| black_box(a == b));
    });
}

fn bench_pose_equality_dynamic(c: &mut Criterion) {
    let a = Pose::from("dynamic-equality-test");
    let b = Pose::from("dynamic-equality-test");

    c.bench_function("equality (dynamic)", |bencher| {
        bencher.iter(|| black_box(a == b));
    });
}

fn bench_pose_as_str_static(c: &mut Criterion) {
    let p = pose!("color");

    c.bench_function("as_str (static)", |bencher| {
        bencher.iter(|| black_box(p.as_str()));
    });
}

fn bench_pose_as_str_dynamic(c: &mut Criterion) {
    let p = Pose::from("dynamic-as-str-test");

    c.bench_function("as_str (dynamic)", |bencher| {
        bencher.iter(|| black_box(p.as_str()));
    });
}

fn bench_pose_hash_static(c: &mut Criterion) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let p = pose!("color");

    c.bench_function("hash (static)", |bencher| {
        bencher.iter(|| {
            let mut hasher = DefaultHasher::new();
            p.hash(&mut hasher);
            black_box(hasher.finish())
        });
    });
}

fn bench_pose_hash_dynamic(c: &mut Criterion) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let p = Pose::from("dynamic-hash-test");

    c.bench_function("hash (dynamic)", |bencher| {
        bencher.iter(|| {
            let mut hasher = DefaultHasher::new();
            p.hash(&mut hasher);
            black_box(hasher.finish())
        });
    });
}

fn bench_hashmap_insert_lookup(c: &mut Criterion) {
    use std::collections::HashMap;

    c.bench_function("HashMap<Pose, _> insert + lookup", |bencher| {
        bencher.iter(|| {
            let mut map: HashMap<Pose, u32> = HashMap::new();

            map.insert(pose!("color"), 1);
            map.insert(pose!("display"), 2);
            map.insert(pose!("margin-top"), 3);
            map.insert(Pose::from("--custom-prop"), 4);

            black_box(map.get(&pose!("color")));
            black_box(map.get(&pose!("display")));
            black_box(map.get(&Pose::from("--custom-prop")));
        });
    });
}

criterion_group!(
    benches,
    bench_static_pose_macro,
    bench_static_pose_from,
    bench_dynamic_pose_first,
    bench_dynamic_pose_cached,
    bench_pose_equality_static,
    bench_pose_equality_dynamic,
    bench_pose_as_str_static,
    bench_pose_as_str_dynamic,
    bench_pose_hash_static,
    bench_pose_hash_dynamic,
    bench_hashmap_insert_lookup,
);

criterion_main!(benches);
