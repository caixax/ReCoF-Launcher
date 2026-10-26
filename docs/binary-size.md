# Binary size

The launcher is a single executable that people download and run, often after being told by a friend that it is safe. A small one is easier to trust, quicker to fetch and gives antivirus heuristics less to chew on. This records what was measured rather than what was assumed.

All figures are `ReCoF-Launcher.exe` built with

```sh
cargo build --release --target x86_64-pc-windows-msvc
```

on the release profile in [`Cargo.toml`](../Cargo.toml): fat LTO, one codegen unit, `panic = "abort"`, symbols stripped, and the static CRT from [`.cargo/config.toml`](../.cargo/config.toml).

## The current size

| | Bytes | |
|---|---:|---|
| `ReCoF-Launcher.exe` | 7,412,736 | 7.07 MiB |

Measured 2026-10-26 on rustc 1.98.0. No Visual C++ redistributable and no .NET runtime are needed to run it.

## opt-level: "s" beats "z"

The usual advice is that `opt-level = "z"` makes the smallest binary. On this project it does not, so the profile uses `"s"`:

| opt-level | Bytes | Against "s" |
|---|---:|---:|
| `"z"` | 7,739,904 | +327,168 |
| `"s"` | 7,412,736 | baseline |

`"z"` is 4.2 percent **larger**. It differs from `"s"` mainly by also disabling loop vectorization, and for code shaped like this one that turns some tight loops into more instructions rather than fewer. Since `"s"` is both smaller and faster here, there is nothing to trade off.

Worth re-measuring after a large dependency change or a compiler upgrade: the answer is a property of this code and this toolchain, not a general rule.

## What the features cost

| Feature | Bytes | |
|---|---:|---|
| eframe `system_fonts` | 416,256 | 406 KiB |

`system_fonts` loads a font already installed on the system when the built in fonts have no glyph for a character. Without it the Simplified Chinese translation draws as empty boxes, and so does its own name in the language list, which is the entry a Chinese speaker has to recognise before they can choose it.

The alternative considered was embedding a CJK font. Microsoft YaHei is about 19 MB and a subset built from the current translation would be a few hundred KB but would have to be regenerated every time the text changes. 406 KiB for a fallback that covers every script, and that keeps working when a translation grows, is the better deal.

## What the size is actually made of

Most of it is egui and its dependencies: the OpenGL backend, winit, the font rasteriser and the system font provider. The launcher's own logic, the eleven translation files and the Windows bindings are a small part of the total. Replacing the UI toolkit is the only change that would move the number significantly, and it is not worth it.

## What is deliberately not done

- **No UPX or any other packer.** A packed executable that downloads DLLs and writes an `opengl32.dll` into a game folder is close to the textbook description of what antivirus heuristics look for. The feasibility study makes the same point.
- **No self-extracting wrapper**, for the same reason.
- **No stripping of the translations.** All eleven languages are embedded. They are plain text and compress to very little.

## Measuring it again

```sh
cargo build --release --target x86_64-pc-windows-msvc
ls -l target/x86_64-pc-windows-msvc/release/ReCoF-Launcher.exe
```

CI prints the size in the job summary on every push, so a change that makes the executable noticeably bigger shows up in the pull request rather than in a release.
