# Stiqka

Stiqka (old StickerFast WA) is a desktop app to make sticker packs for WhatsApp, built in Rust.

- state: Development
- rust version: 1.99.0
- cargo version: 1.99.0
- built on the OS: Win 11

*If you liked this project give me a star*

## ℹ️ Notes

Unfortunately WhatsApp doesn't provide a native way to import packs from a file, so we depend on third-party app methods to import them. We create the packs with the `.wastickers` extension; as a user, you are free to use the app you prefer. For testing, we use Sticker Maker.

## 🚀 Getting started

### Prerequisites

Make sure you have Rust and Cargo installed:

```bash
rustc --version
cargo --version
```

If you don't have Rust, install it from [rustup.rs](https://rustup.rs/).

### Clone and run

```bash
# Clone the repository
git clone https://github.com/your-user/sfwa.git
cd sfwa

# Build the project
cargo build

# Run the app
cargo run

# Run tests
cargo test

# Check for lints
cargo clippy

# Format the code
cargo fmt
```

### Project structure

```
sfwa/
├── Cargo.toml          # Manifest with dependencies
├── Cargo.lock          # Locked dependency versions
├── LICENSE             # GNU AGPLv3 license text
├── README.md           # This file
├── src/
│   ├── main.rs         # Application entry point
│   ├── lib/            # Core library (image manipulation, packing)
│   │   ├── images.rs   # ImageManipulator: resize, thumbnail, animated WebP, etc.
│   │   └── packer.rs   # Packer: creates .wastickers ZIP archives
│   ├── logic/          # Business logic
│   │   ├── main.rs     # create_pack, process_images, image transforms
│   │   ├── controller.rs # Progress/result state for the UI
│   │   └── models.rs   # PackInfo data model
│   ├── view/           # egui UI layer
│   │   └── mod.rs      # StiqkaApp UI, forms, progress bar, toast
│   └── translations/   # i18n JSON files (en, es, pt, de, fr, it)
└── assets/             # Fonts, icons, test images
```

### Building a release

```bash
cargo build --release
```

The binary will be available at `target/release/Stiqka` (or `Stiqka.exe` on Windows).

## ⚖️ License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0-only)**.

See the [LICENSE](LICENSE) file for the full license text.

### Why AGPLv3?

- **Strong copyleft:** Anyone who modifies and distributes this software must release their changes under the same license.
- **Protection against network use without sharing code:** If someone takes the `logic` and `lib` code, modifies it, and exposes it as a network service (e.g., a web server), they must publish the full source code of their modifications to the users of that service.
- **Goal:** Prevent someone from taking the tool, closing it, and profiting commercially without giving the changes back to the community.

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.