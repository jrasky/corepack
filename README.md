# corepack
A no_std support for messagepack in serde

[Documentation](https://docs.rs/corepack)

[MPL 2.0 License](LICENSE)

To use:
```toml
corepack = "0.1"
```

Note: if you want to use corepack with a `std` serde, enable the `std` feature.

```toml
corepack = { version = "0.1", features = ["std"] }
```

Note: this package uses serde 0.8, and so requires patches to serde to be able
to use the `#[derive(Serialize, Deserialize)]` successfully in certain
situations in a `no_std` environment. Changes to update it to serde 0.9 are
forthcoming.
