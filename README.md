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

These two commands kill the built-in DNS server on ubuntu. Which is nice b/c i can run on port 53, but then the crawler can do bootstrap from existing DNS seeds!

```
# systemctl disable systemd-resolved
# systemctl stop systemd-resolved
```

To get this working I had to create an `NS` record pointing from `seed.justinmoon.com` -> `dnsseed.justinmoon.com` and an `A` record pointing from `dnsseed.justinmoon.com` to the IP where this crawler is running.

I also changed the host in /etc/resolv to the ip the crawler was deployed to
