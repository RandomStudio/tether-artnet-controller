# Tether ArtNet Controller

Example: start with connected interface:
```
cargo run --release -- --artnet.interface 10.0.0.100 --artnet.destination 10.0.0.99
```

Example: start with local testing for ArtnetView:
```
cargo run -- --artnet.interface 10.112.10.187 --artnet.destination 10.112.10.187 --loglevel debug
```

## TODO
- [ ] Add labels for which channels are already "taken"/assigned in master slider list
- [ ] With macros, add some visual indicators of state, e.g. Colour, Brightness and Pan/Tilt 
- [ ] Add indications of "range values" for channels where this applies (under fixture section)
- [ ] Add the missing macros/auto sections with range values for both fixture types (left out this detail)
- [ ] Add 16-bit control, at least for macros (single slider adjusts the two channels as split between first and second 8-bit digits)
