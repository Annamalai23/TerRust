use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use terrust::terminal::AnsiParser;

fn parser_plain_text(c: &mut Criterion) {
    let text = b"Hello, World! This is a simple text without any ANSI escape sequences.\n";
    let mut group = c.benchmark_group("ansi_parser");
    group.throughput(Throughput::Bytes(text.len() as u64));
    group.bench_function("plain_text", |b| {
        b.iter(|| {
            let mut parser = AnsiParser::new();
            for &byte in black_box(text) {
                black_box(parser.parse(byte));
            }
        });
    });
    group.finish();
}

fn parser_sgr_bold_red(c: &mut Criterion) {
    let seq = b"\x1b[1;31mBold Red Text\x1b[0mReset\n";
    let mut group = c.benchmark_group("ansi_parser");
    group.throughput(Throughput::Bytes(seq.len() as u64));
    group.bench_function("sgr_bold_red", |b| {
        b.iter(|| {
            let mut parser = AnsiParser::new();
            for &byte in black_box(seq) {
                black_box(parser.parse(byte));
            }
        });
    });
    group.finish();
}

fn parser_complex_sgr(c: &mut Criterion) {
    let seq = b"\x1b[38;5;82m\x1b[48;5;17m\x1b[1;4mStyled Text\x1b[0m";
    let mut group = c.benchmark_group("ansi_parser");
    group.throughput(Throughput::Bytes(seq.len() as u64));
    group.bench_function("complex_sgr", |b| {
        b.iter(|| {
            let mut parser = AnsiParser::new();
            for &byte in black_box(seq) {
                black_box(parser.parse(byte));
            }
        });
    });
    group.finish();
}

fn parser_cursor_movement(c: &mut Criterion) {
    let seq = b"\x1b[10A\x1b[5B\x1b[20C\x1b[3D\x1b[1;1H";
    let mut group = c.benchmark_group("ansi_parser");
    group.throughput(Throughput::Bytes(seq.len() as u64));
    group.bench_function("cursor_movement", |b| {
        b.iter(|| {
            let mut parser = AnsiParser::new();
            for &byte in black_box(seq) {
                black_box(parser.parse(byte));
            }
        });
    });
    group.finish();
}

fn parser_erase_sequences(c: &mut Criterion) {
    let seq = b"\x1b[2J\x1b[K\x1b[1J\x1b[0J\x1b[1K\x1b[0K";
    let mut group = c.benchmark_group("ansi_parser");
    group.throughput(Throughput::Bytes(seq.len() as u64));
    group.bench_function("erase_sequences", |b| {
        b.iter(|| {
            let mut parser = AnsiParser::new();
            for &byte in black_box(seq) {
                black_box(parser.parse(byte));
            }
        });
    });
    group.finish();
}

fn parser_osc_sequences(c: &mut Criterion) {
    let seq = b"\x1b]0;My Terminal Window\x07\x1b]8;;https://example.com\x07\x1b]8;;\x07";
    let mut group = c.benchmark_group("ansi_parser");
    group.throughput(Throughput::Bytes(seq.len() as u64));
    group.bench_function("osc_sequences", |b| {
        b.iter(|| {
            let mut parser = AnsiParser::new();
            for &byte in black_box(seq) {
                black_box(parser.parse(byte));
            }
        });
    });
    group.finish();
}

fn parser_alternate_screen(c: &mut Criterion) {
    let seq = b"\x1b[?1049h\x1b[?25l\x1b[2J\x1b[?25h\x1b[?1049l";
    let mut group = c.benchmark_group("ansi_parser");
    group.throughput(Throughput::Bytes(seq.len() as u64));
    group.bench_function("alternate_screen", |b| {
        b.iter(|| {
            let mut parser = AnsiParser::new();
            for &byte in black_box(seq) {
                black_box(parser.parse(byte));
            }
        });
    });
    group.finish();
}

fn parser_mixed_content(c: &mut Criterion) {
    let content = b"Normal text \x1b[1mBOLD\x1b[0m and \x1b[31mred\x1b[0m and \x1b[4munderline\x1b[0m.\n\x1b[32mGreen line\x1b[0m with \x1b[1;33mBold Yellow\x1b[0m.\n\x1b[1;4;31mBold Underline Red\x1b[0m\n";
    let mut group = c.benchmark_group("ansi_parser");
    group.throughput(Throughput::Bytes(content.len() as u64));
    group.bench_function("mixed_content", |b| {
        b.iter(|| {
            let mut parser = AnsiParser::new();
            for &byte in black_box(content) {
                black_box(parser.parse(byte));
            }
        });
    });
    group.finish();
}

fn parser_reset_reuse(c: &mut Criterion) {
    let seq = b"\x1b[1;31mHello\x1b[0m";
    let mut group = c.benchmark_group("ansi_parser");
    group.bench_function("reset_reuse", |b| {
        b.iter(|| {
            let mut parser = AnsiParser::new();
            for _ in 0..100 {
                for &byte in black_box(seq) {
                    black_box(parser.parse(byte));
                }
                parser.reset();
            }
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    parser_plain_text,
    parser_sgr_bold_red,
    parser_complex_sgr,
    parser_cursor_movement,
    parser_erase_sequences,
    parser_osc_sequences,
    parser_alternate_screen,
    parser_mixed_content,
    parser_reset_reuse,
);
criterion_main!(benches);
