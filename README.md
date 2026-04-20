[![Rust](https://github.com/tomBoddaert/dyn-slice/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/tomBoddaert/dyn-slice/actions/workflows/rust.yml)

# Dyn-Slice

An implementation of a `&[dyn Trait]`-like reference, inspired by a [Reddit thread](https://www.reddit.com/r/rust/comments/14i08gz/dyn_slices).

Dyn-slices are slices of trait objects. Indexing into one yields a trait object reference. The vtable pointer is only stored once.

```sh
cargo add dyn-slice
```

[Latest documentation](https://docs.rs/dyn-slice/latest/dyn_slice/)  
[Examples](/examples/)

[dyn-slice on crates.io](https://crates.io/crates/dyn-slice)  
[dyn-slice on lib.rs](https://lib.rs/crates/dyn-slice)  
[dyn-slice on GitHub](https://github.com/tomBoddaert/dyn-slice)

## License

[Dyn-Slice](https://github.com/tomBoddaert/dyn-slice) is dual-licensed under either the MIT license or Apache License Version 2.0 at your option.
