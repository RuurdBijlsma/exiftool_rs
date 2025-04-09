use criterion::{black_box, criterion_group, criterion_main, Criterion};
use exiftool::ExifTool;
use serde_json::Value;
use std::path::Path;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut et = ExifTool::new()?;
    let _: u32 = et.read_tag(Path::new("data/image.jpg"), "ImageWidth")?;
    Ok(())
}

fn bench_exiftool(c: &mut Criterion) {
    c.bench_function("spawn & read", |b| b.iter(|| run()));

    let mut et = ExifTool::new().expect("Failed to spawn ExifTool");
    c.bench_function("just read_tag", |b| {
        b.iter(|| {
            let _: u32 = et
                .read_tag(
                    black_box(Path::new("data/image.jpg")),
                    black_box("ImageWidth"),
                )
                .unwrap();
        })
    });

    c.bench_function("full binary output", |b| {
        b.iter(|| {
            let _: Vec<u8> = et.execute_raw(&[black_box("data/image.jpg")]).unwrap();
        })
    });

    c.bench_function("full json output", |b| {
        b.iter(|| {
            let _: Value = et
                .json(black_box(Path::new("data/image.jpg")), &[])
                .unwrap();
        })
    });
}

criterion_group!(benches, bench_exiftool);
criterion_main!(benches);
