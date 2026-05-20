use crate::format_source;

#[test]
fn formats_minimal() {
    let src = "let x = 1 + 2;\nprint x;\n";
    let out = format_source(src).expect("format");
    assert!(out.contains("let x = 1 + 2;"));
    assert!(out.contains("print x;"));
}

#[test]
fn preserves_line_comment() {
    let src = "// header\nlet x = 1;\n";
    let out = format_source(src).expect("format");
    assert!(out.contains("// header"));
    assert!(out.contains("let x = 1;"));
}

#[test]
fn preserves_trailing_comment() {
    let src = "let x = 1; // value\n";
    let out = format_source(src).expect("format");
    assert!(out.contains("let x = 1;"));
    assert!(out.contains("// value"));
}

#[test]
fn preserves_inline_block_comment() {
    let src = "let y = 1 + /* offset */ 2;\n";
    let out = format_source(src).expect("format");
    assert!(out.contains("/* offset */"));
}

#[test]
fn idempotent() {
    let src = "// header\nlet x = 1 + 2; // sum\nlet y = x * /* offset */ 10;\nprint y; // out\n";
    let once = format_source(src).expect("first");
    let twice = format_source(&once).expect("second");
    assert_eq!(once, twice);
}
