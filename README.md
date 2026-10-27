<p align="center">
  <img src="https://raw.githubusercontent.com/Canary-ECAD/.github/main/assets/canary-ecad.png" width="140" alt="Canary E-cad"/>
</p>

# open-ecad

**Open-source electrical CAD, built in Rust.** The domain model is derived
**clean-room** from public documentation and open standards (IEC 81346, IEC 60617):
no reverse engineering, no third-party code or assets, nothing copied from any vendor.
The design targets the workflow of industrial E-CAD suites (EPLAN Electric P8 class)
without their noise: fewer dialogs, fewer configuration layers, the same engineering power.

<p align="center">
  <a href="#license"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-F5B921?style=flat-square" alt="License"/></a>
  <a href="https://github.com/Canary-ECAD/open-ecad/actions/workflows/ci.yml"><img src="https://github.com/Canary-ECAD/open-ecad/actions/workflows/ci.yml/badge.svg" alt="CI"/></a>
  <a href="https://github.com/Canary-ECAD/open-ecad/discussions"><img src="https://img.shields.io/badge/chat-discussions-14212B?style=flat-square" alt="Discussions"/></a>
  <a href="CONTRIBUTING.md"><img src="https://img.shields.io/badge/contributions-welcome-F26419?style=flat-square" alt="Contributing"/></a>
</p>

---

## Status

🚧 **Design phase.** The full domain model is mapped in
[`MAPA-DOMINIO.md`](MAPA-DOMINIO.md) *(currently Spanish only; English translation on the
roadmap)*. The Rust workspace compiles and CI is green from day one — the core crate is a
scaffold waiting for the domain types.

The user interface is mapped in [`MAPA-UI.md`](MAPA-UI.md) *(Spanish)*: what a working
designer actually uses, what gets merged or dropped, and the v1 window skeleton.

## Why "open-box"?

Because you can audit the method, not just the code:

- The domain map is public and explains **where every concept came from** — open standards
  and public API documentation, described in our own words.
- The vendor documentation used as *background reading* lives in a gitignored folder and is
  **never distributed** with the project.
- Every PR must re-affirm the [clean-room pledge](CONTRIBUTING.md#the-clean-room-pledge).

## Roadmap

| Phase | Scope |
|---|---|
| **v1** | 2D schematic: functions + symbols, connection engine derived from topology, potentials and cross-references between pages, open symbol library (SVG + JSON), macros with placeholders, transactional undo, PDF/SVG export |
| **v2** | Article database, electrical checks, renumbering, reports, DXF/DWG |
| **v3** | AML (AutomationML), multi-user, revisions |

## Repository layout

```
crates/
  open-ecad-core/      # domain model — GUI-agnostic crate
  open-ecad-app/       # desktop app (egui) — window prototype, mock data
.github/
  ISSUE_TEMPLATE/      # bug & feature forms
  workflows/ci.yml     # fmt · clippy · test
MAPA-DOMINIO.md        # clean-room domain map (design spec)
MAPA-UI.md             # UI map: real-world usage, verdicts, v1 skeleton
CONTRIBUTING.md        # how to help + the clean-room pledge
CODE_OF_CONDUCT.md     # Contributor Covenant 2.1
```

## Window prototype

`crates/open-ecad-app` is the v1 UI skeleton (MAPA-UI.md §4) running on mock data, with no
domain model yet:

```
cargo run -p open-ecad-app
```

- CAD-style ribbon (tabs, named groups, large/small buttons), shortcuts and a command
  palette (`Ctrl+K`), all generated from a single command registry. Greyed-out commands
  have a place in the UI but do nothing yet.
- The drawing area is fixed in the centre and shows one page at a time; clicking a page in
  the navigator replaces it (`PgUp` / `PgDn` for previous/next).
- Dockable panels in three zones (left, right, bottom): drag and stack within a zone;
  right-click a tab → *Mover a* to change zone. The layout is remembered on exit.
- Canvas: wheel = zoom at cursor, middle or right drag = pan, `Home` = fit page,
  `F7` grid, `F9` snap.

## Planned stack

Rust + [egui](https://github.com/emilk/egui) (immediate-mode canvas, docking, native +
WASM). The domain model stays a standalone crate, independent of the GUI, so scripting
and headless tooling can use it without any window.

## Get involved

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) first — the clean-room pledge is the one hard
rule. Design debates belong in [Discussions](https://github.com/Canary-ECAD/open-ecad/discussions);
the domain map is where the real decisions happen.

## License

Dual-licensed, at your option: [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE)
(the Rust ecosystem convention).

---

<sub>open-ecad is an independent project and is not affiliated with, endorsed by, or
sponsored by any E-CAD platform vendor. EPLAN® is a registered trademark of its owner
and is mentioned here only to describe the class of workflows this project targets.</sub>
