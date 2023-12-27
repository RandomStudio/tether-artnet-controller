# Tether ArtNet Controller

A reasonably generic ArtNet control system with the following features:
- Software-based "Lighting Desk" for manually manipulating light fixtures
- Controllable via a MIDI controller
- Controllable via Tether "remote commands" including animations

## CLI

Example: start with connected interface:
```
cargo run --release -- --artnet.interface 10.0.0.100 --artnet.destination 10.0.0.99
```

Example: start with local testing for ArtnetView:
```
cargo run -- --artnet.interface 10.112.10.187 --artnet.destination 10.112.10.187 --loglevel debug
```

## Remote control
If you have Tether Egui installed (`cargo install tether-egui`) then the easiest way to test Tether remote control is to launch Tether Egui with the example project file included, i.e.:
`tether-egui tether-egui-testing.json`

## TODO
- [x] Add labels for which channels are already "taken"/assigned in master slider list
- [x] Add indications of "range values" for channels where this applies (under fixture section)
- [x] Allow macros to be (temporarily) disabled so Fixture>Mapping values can be adjusted directly without being overridden
- [x] Add the missing macros/auto sections with range values for both fixture types (left out this detail)
- [x] MIDI (Tether, remote) control
- [x] Allow macro "current values" to be updated remotely via Tether
- [x] Macro Animations on remote messages via Tether
- [ ] Project JSON should save ArtNet configuration (but can override via CLI args)
- [x] Provide simple/advanced views (e.g. "macros only" vs "the kitchen sink")
- [ ] Scenes should be triggered by Tether messages
- [ ] Multi-Macro "cues" (kind of like keyframes: multiple values) should be easy to save. Just hit a button to save the current cue (state of all macros/channels), for example.
- [ ] ArtNet on separate thread, with more precise timing; this might require some messaging back and forth and/or mutex
- [ ] With macros, add some visual indicators of state, e.g. Colour, Brightness and Pan/Tilt 
- [ ] Need to resolve "conflict" between values that have defaults but ALSO have Macros attached
- [ ] Add 16-bit control, at least for macros (single slider adjusts the two channels as split between first and second 8-bit digits)
