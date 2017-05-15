# corepack
A better messagepack implementation for serde

[Documentation](https://docs.rs/corepack)

[MPL 2.0 License](LICENSE)

To use:
```toml
corepack = "~0.2.0"
```

If you want to use corepack in a `no_std` environment (nightly rust required),
disable the "std" feature and enable the "collections" feature:

```toml
corepack = { version = "~0.2.0", default-features = false, features = ["collections"] }
```

You _must_ choose either "std" or "collections" as a feature. Corepack currently
requires dynamic allocations in a few situations.