# Contributing to open-ecad

Thanks for your interest. open-ecad is in **design phase** — the highest-value
contributions right now are arguments about the domain model, not code.

## The clean-room pledge

This is the one hard rule, and it is what makes the project legally safe to exist:

> All work in this repository is derived **exclusively from public documentation, open
> standards (IEC 81346, IEC 60617, …) and general engineering knowledge**, written in our
> own words. **No reverse engineering. No vendor code, assets, symbol libraries, project
> files, exports, or proprietary documentation** — not copied, not translated, not
> decompiled, not "lightly adapted".

Practically, that means:

1. **Never commit vendor documentation**, even as `docs/` reference. The reading folder
   (`_docs-eplan/`) is gitignored on purpose. Keep it out of the repo.
2. Descriptions of third-party concepts (what a *connection point* is, how device
   identification is structured) are fine — that's domain knowledge published in standards
   and help wikis. Verbatim text or data dumps from anywhere are not.
3. Symbol libraries are **drawn from IEC 60617**, not traced from any vendor's library.
4. If you are unsure whether something crosses the line — open a discussion *before*
   writing it.

Every PR confirms this pledge in its checklist.

## Development setup

```
rustup toolchain install stable        # rust-toolchain.toml pins fmt + clippy
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

CI runs exactly those three gates.

## Where to start

- **Domain model** (`crates/open-ecad-core`): the map's §2 decisions are the work order —
  structured device identifiers (`=plant+location-product`), functions vs. symbol
  references, the derived-connection engine.
- **Docs**: translation and critique of `MAPA-DOMINIO.md` are first-class contributions.
- Check the issue tracker for `good first issue` labels as they appear.

## Process

- Open an issue or discussion before large changes; design debates in the open leave a
  trail this project will need for years.
- Use the PR template; keep the three CI gates green; one concern per PR.
- We follow the [Code of Conduct](CODE_OF_CONDUCT.md). Report issues to the maintainers
  via a private GitHub security advisory or by messaging an org owner.

## Commit & style

- Rustfmt defaults; clippy warnings are errors.
- Imperative commit subjects, keep them under ~70 chars.
- Do not include personal metadata, session links, or local paths in commits.
