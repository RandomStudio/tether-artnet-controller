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
- [x] Add labels for which channels are already "taken"/assigned in master slider list
- [x] Add indications of "range values" for channels where this applies (under fixture section)
- [x] Allow macros to be (temporarily) disabled so Fixture>Mapping values can be adjusted directly without being overridden
- [x] Add the missing macros/auto sections with range values for both fixture types (left out this detail)
- [x] MIDI (Tether, remote) control
- [ ] Need to resolve "conflict" between values that have defaults but ALSO have Macros attached
- [ ] Allow macro "current values" to be updated remotely via Tether
- [ ] Macro Animations on remote messages via Tether
- [ ] ArtNet on separate thread, with more precise timing; this might require some messaging back and forth and/or mutex
- [ ] Remote control for macros
- [ ] With macros, add some visual indicators of state, e.g. Colour, Brightness and Pan/Tilt 
- [ ] Multi-Macro "cues" (kind of like keyframes: multiple values) should be easy to save. Just hit a button to save the current cue (state of all macros/channels), for example.
- [ ] Add 16-bit control, at least for macros (single slider adjusts the two channels as split between first and second 8-bit digits)
