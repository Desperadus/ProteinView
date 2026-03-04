# ProteinView

```
╔══════════════════════════════════════════════════════════════════════════╗
║                                                                        ║
║  ██████╗ ██████╗  ██████╗ ████████╗███████╗██╗███╗   ██╗              ║
║  ██╔══██╗██╔══██╗██╔═══██╗╚══██╔══╝██╔════╝██║████╗  ██║              ║
║  ██████╔╝██████╔╝██║   ██║   ██║   █████╗  ██║██╔██╗ ██║              ║
║  ██╔═══╝ ██╔══██╗██║   ██║   ██║   ██╔══╝  ██║██║╚██╗██║              ║
║  ██║     ██║  ██║╚██████╔╝   ██║   ███████╗██║██║ ╚████║              ║
║  ╚═╝     ╚═╝  ╚═╝ ╚═════╝    ╚═╝   ╚══════╝╚═╝╚═╝  ╚═══╝              ║
║              ██╗   ██╗██╗███████╗██╗    ██╗                             ║
║              ██║   ██║██║██╔════╝██║    ██║                             ║
║              ██║   ██║██║█████╗  ██║ █╗ ██║                             ║
║              ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║                             ║
║               ╚████╔╝ ██║███████╗╚███╔███╔╝                             ║
║                ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝                              ║
║                                                                        ║
║     (=(    )=)~~(=(    )=)~~(=(    )=)~~(=(    )=)~~(=(    )=)         ║
║                                                                        ║
╚══════════════════════════════════════════════════════════════════════════╝
```

Terminal protein structure viewer -- load, rotate, and explore PDB/CIF structures right in your terminal.

## Features

- **Braille character rendering** -- works everywhere, including over SSH
- **HD pixel mode** -- sixel/kitty graphics for capable terminals (`--hd`)
- **Interactive rotation, zoom, pan** with vim-style keybindings
- **5 color schemes** -- secondary structure, chain, element, B-factor, rainbow
- **PDB and mmCIF format support** via pdbtbx
- **Single static binary**, zero runtime dependencies

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# View a local PDB file
proteinview examples/1UBQ.pdb

# HD pixel mode (sixel/kitty terminals)
proteinview examples/4HHB.pdb --hd

# Fetch from RCSB PDB (requires --features fetch)
proteinview --fetch 1UBQ
```

## Keybindings

| Key       | Action               |
|-----------|----------------------|
| `h` / `l` | Rotate Y-axis        |
| `j` / `k` | Rotate X-axis        |
| `u` / `i` | Rotate Z-axis (roll) |
| `+` / `-` | Zoom in / out        |
| `w/a/s/d` | Pan                  |
| `r`       | Reset view           |
| `c`       | Cycle color scheme   |
| `v`       | Cycle visualization mode |
| `m`       | Toggle braille / HD  |
| `[` / `]` | Previous / next chain |
| `Space`   | Toggle auto-rotation |
| `?`       | Help overlay         |
| `q`       | Quit                 |

## Color Schemes

| Scheme               | Description                                                        |
|----------------------|--------------------------------------------------------------------|
| **Secondary Structure** | Helix (red), sheet (yellow), coil (green), turn (blue). Default. |
| **Chain**            | Each chain gets a distinct color from a curated palette.           |
| **Element (CPK)**    | Atoms colored by element: C gray, N blue, O red, S yellow.        |
| **B-factor**         | Blue (low mobility) to red (high mobility) gradient.               |
| **Rainbow**          | N-terminus (blue) to C-terminus (red) by residue position.         |

## Example PDB Files

| File                  | Description                                                 |
|-----------------------|-------------------------------------------------------------|
| `examples/1UBQ.pdb`  | Ubiquitin -- 76 residues, single chain, classic test protein |
| `examples/4HHB.pdb`  | Hemoglobin -- 4 chains, 574 residues, good for multi-chain viewing |

## Building

```bash
cargo build --release

# With RCSB fetch support:
cargo build --release --features fetch
```

## License

MIT
