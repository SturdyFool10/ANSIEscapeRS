# ANSIEscapeRS

**ANSIEscapeRS** is a Rust library for generating, parsing, and working with ANSI escape codes. It provides a type-safe, extensible API for producing and interpreting ANSI codes for text formatting, color, cursor movement, and terminal control, with a focus on making invalid states unrepresentable.

---

## Features

- **Type-safe ANSI code generation**: Use enums and builder patterns to create valid ANSI escape sequences.
- **Parsing and interpretation**: Efficiently parse strings containing ANSI codes into structured representations.
- **Environment detection**: Query terminal capabilities (ANSI support, truecolor, 8-bit color, etc.).
- **Comprehensive color support**: Standard, 8-bit, and 24-bit (truecolor) modes.
- **Cursor and device control**: Move the cursor, clear the screen, and more.
- **Tested**: Includes extensive unit tests for formatting and parsing.

---

## Modules Overview

### `ansi_creator` (accessed via crate root)

- **Purpose**: API for producing ANSI escape codes and querying environment capabilities.
- **Key Types**:
  - `AnsiEnvironment`: Detects terminal support for ANSI, truecolor, and 8-bit color.
  - `AnsiCreator`: Main struct for formatting text, generating SGR (Select Graphic Rendition) codes, cursor movement, erase, and device control codes.
- **Example**:
    ```rust
    use ansi_escapers::{creator, SgrAttribute, Color};

    let creator = creator::AnsiCreator::new();
    let bold_red = creator.format_text(
        "Hello",
        &[SgrAttribute::Bold, SgrAttribute::Foreground(Color::Red)]
    );
    println!("{}", bold_red);
    ```

### `interpreter` (accessed via `ansi_escapers::interpreter`)

- **Purpose**: Efficient parser for interpreting ANSI escape codes in strings.
- **Key Types**:
  - `AnsiSpan`: Represents a span of text affected by an ANSI code.
  - `AnsiPoint`: Represents a point event (e.g., cursor move).
  - `AnsiParseResult`: Contains cleaned text, spans, and points.
  - `AnsiParser`: State machine for parsing ANSI codes.
- **Example**:
    ```rust
    use ansi_escapers::interpreter::AnsiParser;

    let mut parser = AnsiParser::new("\x1b[31mRed\x1b[0m Normal");
    let result = parser.parse_annotated();
    println!("{:?}", result.spans);
    ```

### `ansi_types` (accessed via crate root)

- **Purpose**: Core enums representing ANSI escape code capabilities.
- **Key Types**:
  - `SgrAttribute`: Bold, Italic, Underline, Foreground/Background/UnderlineColor, etc.
  - `Color`: Standard, bright, 8-bit, and 24-bit RGB colors.
  - `CursorMove`, `Erase`, `EraseMode`, `DeviceControl`, `AnsiEscape`: All major ANSI command types.

---

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
ansi_escapers = "0.1.0"
```

Import and use in your Rust code (all main types are available from the crate root):

```rust
use ansi_escapers::{creator, SgrAttribute, Color};

let creator = creator::AnsiCreator::new();
let styled = creator.format_text(
    "Hello, world!",
    &[SgrAttribute::Bold, SgrAttribute::Foreground(Color::Blue)]
);
println!("{}", styled);
```

---

## Environment Detection

The library can detect terminal capabilities:

```rust
use ansi_escapers::AnsiEnvironment;

let env = AnsiEnvironment::detect();
println!(
    "ANSI: {}, Truecolor: {}, 8-bit: {}",
    env.supports_ansi, env.supports_truecolor, env.supports_8bit_color
);
```

---

## Testing

Run the tests with:

```sh
cargo test
```

---

## License

This project is licensed under the MIT License.

---

## Contributing

Contributions, issues, and feature requests are welcome! Please open an issue or submit a pull request.

---
