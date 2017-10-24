## Compiling

Either use `cargo run <dir-name>` to use the tool immediately or use `cargo build --release` to let cargo build a binary for you in `target/release`.

## Info

The tool assumes the following structure:

```
<dir-name>
├── tiny_rick
│   ├── grade-prefilled.rktd
│   ├── grade.rktd
│   ├── handin.rkt
│   └── ...
├── ...
```

If you use the `save` and `load` features `turack` places a `turack_progress.txt` in `<dir-name>`. Hopefully this plays nice with Owncloud.
