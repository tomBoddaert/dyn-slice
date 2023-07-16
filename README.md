[![Rust](https://github.com/tomBoddaert/dyn-slice/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/tomBoddaert/dyn-slice/actions/workflows/rust.yml)

# Dyn-Slice

An implementation for a `&dyn [Trait]`-like reference, inspired by a [Reddit thread](https://www.reddit.com/r/rust/comments/14i08gz/dyn_slices).

```sh
cargo add dyn-slice
```

[Latest documentation](https://docs.rs/dyn-slice/latest/dyn_slice/)  
[Examples](/examples/)

[dyn-slice on crates.io](https://crates.io/crates/dyn-slice)  
[dyn-slice on GitHub](https://github.com/tomBoddaert/dyn-slice)

## Warning

DO NOT USE THIS IN PRODUCTION (or any important) CODE: IT IS A PROOF OF CONCEPT AND PROBABLY HAS A FEW BUGS.

With that said, feel free to use it in code that does not have to be reliable and to send pull requests to fix some of the bugs.

## License

[Dyn-Slice](https://github.com/tomBoddaert/dyn-slice) is dual-licensed under either the Apache License Version 2.0 or MIT license at your option.
