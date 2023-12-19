# Tether ArtNet Controller

Example: start with connected interface:
```
cargo run --release -- --artnet.interface 10.0.0.100 --artnet.destination 10.0.0.99
```

Example: start with local testing for ArtnetView:
```
cargo run -- --artnet.interface 10.112.10.187 --artnet.destination 10.112.10.187 --loglevel debug
```