[![Rust](https://github.com/tomBoddaert/dyn-slice/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/tomBoddaert/dyn-slice/actions/workflows/rust.yml)

# Dyn-Slice

An implementation of a `&dyn [Trait]`-like reference, inspired by a [Reddit thread](https://www.reddit.com/r/rust/comments/14i08gz/dyn_slices).

`dyn-slice`s are slices of trait object. Indexing into one yields a trait object reference. The vtable pointer is only stored once.

```sh
cargo add dyn-slice
```

[Latest documentation](https://docs.rs/dyn-slice/latest/dyn_slice/)  
[Examples](/examples/)

[dyn-slice on crates.io](https://crates.io/crates/dyn-slice)  
[dyn-slice on lib.rs](https://lib.rs/crates/dyn-slice)  
[dyn-slice on GitHub](https://github.com/tomBoddaert/dyn-slice)

## Warning

This was originally just a proof of concept and may has some bugs, so it may be best not to use it in production (or any important) code.

With that said, feel free to use it in code that does not have to be reliable and to open an issue if you find any bugs.

## License

[Dyn-Slice](https://github.com/tomBoddaert/dyn-slice) is dual-licensed under either the Apache License Version 2.0 or MIT license at your option.
