use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use terrust::terminal::{Cell, Grid, ScrollbackBuffer};

fn grid_new_small(c: &mut Criterion) {
    c.bench_function("grid_new_80x24", |b| {
        b.iter(|| Grid::new(black_box(80), black_box(24)));
    });
}

fn grid_new_large(c: &mut Criterion) {
    c.bench_function("grid_new_200x100", |b| {
        b.iter(|| Grid::new(black_box(200), black_box(100)));
    });
}

fn grid_scroll_up(c: &mut Criterion) {
    let mut grid = Grid::new(80, 24);
    c.bench_function("grid_scroll_up", |b| {
        b.iter(|| {
            grid.scroll_up();
            black_box(&grid);
        });
    });
}

fn grid_clear(c: &mut Criterion) {
    let mut grid = Grid::new(80, 24);
    c.bench_function("grid_clear", |b| {
        b.iter(|| {
            grid.clear();
            black_box(&grid);
        });
    });
}

fn grid_set_get(c: &mut Criterion) {
    let mut grid = Grid::new(80, 24);
    let cell = Cell::default();
    let mut group = c.benchmark_group("grid");
    group.sample_size(20);
    group.bench_function("set_get", |b| {
        b.iter(|| {
            for col in 0..80u16 {
                for row in 0..24u16 {
                    grid.set(black_box(col), black_box(row), cell.clone());
                    black_box(grid.get(col, row));
                }
            }
        });
    });
    group.finish();
}

fn grid_resize(c: &mut Criterion) {
    let mut grid = Grid::new(80, 24);
    c.bench_function("grid_resize", |b| {
        b.iter(|| {
            grid.resize(black_box(120), black_box(40));
            black_box(&grid);
        });
    });
}

fn scrollback_push(c: &mut Criterion) {
    let mut sb = ScrollbackBuffer::new(10000, 80);
    let line = vec![Cell::default(); 80];
    let mut group = c.benchmark_group("scrollback");
    group.throughput(Throughput::Elements(1));
    group.bench_function("push_line", |b| {
        b.iter(|| {
            sb.push_line(black_box(line.clone()));
        });
    });
    group.finish();
}

fn scrollback_push_bulk(c: &mut Criterion) {
    let mut sb = ScrollbackBuffer::new(10000, 80);
    let line = vec![Cell::default(); 80];
    let mut group = c.benchmark_group("scrollback");
    group.bench_function("push_bulk_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                sb.push_line(black_box(line.clone()));
            }
        });
    });
    group.finish();
}

fn scrollback_pop_front(c: &mut Criterion) {
    let mut sb = ScrollbackBuffer::new(5000, 80);
    let line = vec![Cell::default(); 80];
    for _ in 0..5000 {
        sb.push_line(line.clone());
    }
    let mut group = c.benchmark_group("scrollback");
    group.bench_function("pop_front", |b| {
        b.iter(|| {
            if sb.len() > 0 {
                black_box(sb.pop_front());
            }
        });
    });
    group.finish();
}

criterion_group!(
    grid_benches,
    grid_new_small,
    grid_new_large,
    grid_scroll_up,
    grid_clear,
    grid_set_get,
    grid_resize,
    scrollback_push,
    scrollback_push_bulk,
    scrollback_pop_front,
);
criterion_main!(grid_benches);
