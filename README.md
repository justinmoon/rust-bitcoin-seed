# bitcoin-seed

## Run

First, [install Rust](https://www.rust-lang.org/tools/install)

Then, run it with cargo:

```
$ cargo run
```

For more debug output:

```
$ RUST_LOG=trace cargo run
```

## Deploy

I can currently deploy to an ip that points to `seed.justinmoon.com` and this works:

```
dig @seed.justinmoon.com -p 2053 seed.justinmoon.com
```

Need to figure out how to properly run on port 53. Perhaps I need certs ...
