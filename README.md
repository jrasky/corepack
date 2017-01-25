# corepack
A no_std support for messagepack in serde

[Documentation](https://docs.rs/corepack)

To use:
```toml
corepack = "0.1"
```

Note: if you want to use corepack with a `std` serde, enable the `std` feature.

```toml
corepack = { version = "0.1", features = ["std"] }
```

Note that serde support for `#[derive(Serialize, Deserialize)]` is broken
because it generates code that directly links to `std::string`, rather than
`collections::string`. Unfortunately, `collections` is unstable, and so not
usable on stable builds. The solution I've used is to patch the library mysef
when I wanted to use serde code generation in a `no_std` environment, which is
relatively simple.
