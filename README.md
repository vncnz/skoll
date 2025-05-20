# Skoll
Skoll is, in norse mythology, a wolf who chases the sun, causing eclipses.

Please note that this is a personal project, for personal use, developed in my (not so much) free time. You'll not find clean code or a flexible, modular system here. You'll find lots of experiments, abandoned ideas, dead code, temporary hacks and workarounds. Oh, and last but not least, I'm just learning both Rust and GTK. You've been warned.

### Initially copied from https://github.com/DorianRudolph/sirula
This project started as a copy of Sirula project by Dorian Rudolph. I think I'll heavely rewrite/modify the code to archieve what I have in mind but I needed a starting point because I've never developed anything related to Wayland nor in Rust.

## Known bugs and missing features
- [x] Open apps must be immediately recognizable and monitor/workspace infos in the row
- [x] Open apps don't have to open a new instance but focus the existing one
- [x] Open GitHub provokes panic, it seems to be about the app_id being the display_name
- [x] Fullscreen background and UI refactoring
- [ ] PageUp and PageDown keys? Home and End keys?
- [ ] Dedicated class for taskbar voices?
- [ ] Remove fixed size for fullscreen
- [ ] Create the possibility of custom commands passed by as arguments -- study the compatibility with fuzzel et similia for potential replacement
- [ ] Network info?
- [ ] Avg load?
- [ ] Random sentence/quote from a list in toml configuration?
- [ ] Generic input from external source?