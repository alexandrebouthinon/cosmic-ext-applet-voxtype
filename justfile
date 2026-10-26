name := 'cosmic-ext-applet-voxtype'
appid := 'io.github.alexandrebouthinon.CosmicExtAppletVoxtype'

rootdir := ''
prefix := '/usr'

appdata := appid + '.metainfo.xml'
desktop := appid + '.desktop'

# Installation paths
base-dir := absolute_path(clean(rootdir / prefix))
cargo-target-dir := env('CARGO_TARGET_DIR', 'target')
appdata-dst := base-dir / 'share' / 'appdata' / appdata
bin-dst := base-dir / 'bin' / name
desktop-dst := base-dir / 'share' / 'applications' / desktop
icon-dst := base-dir / 'share' / 'icons' / 'hicolor' / 'scalable' / 'apps' / appid + '.svg'

# Default recipe which runs `just build-release`
default: build-release

# Runs `cargo clean`
clean:
    cargo clean

# Removes vendored dependencies
clean-vendor:
    rm -rf .cargo vendor vendor.tar

# `cargo clean` and removes vendored dependencies
clean-dist: clean clean-vendor

# Compiles with debug profile
build-debug *args:
    cargo build {{args}}

# Compiles with release profile
build-release *args: (build-debug '--release' args)

# Compiles release profile with vendored dependencies
build-vendored *args: vendor-extract (build-release '--frozen --offline' args)

# Runs a clippy check
check *args:
    cargo clippy --all-features {{args}} -- -W clippy::pedantic

# Runs a clippy check with JSON message format
check-json: (check '--message-format=json')

# Format, autofix lints, then fail on remaining warnings and run tests
verify:
    cargo fmt
    cargo clippy --fix --allow-dirty --allow-staged --all-features -- -W clippy::pedantic
    cargo clippy --all-features -- -W clippy::pedantic -D warnings
    cargo test

# Installs into ~/.local so the panel can load the applet without root
install-user: build-release
    just prefix={{ env('HOME') / '.local' }} install
    # cosmic-panel's PATH lacks ~/.local/bin: point Exec at the binary itself.
    sed -i 's|^Exec=[^ ]*|Exec={{ env('HOME') / '.local' / 'bin' / name }}|' {{ env('HOME') / '.local' / 'share' / 'applications' / appid + '.desktop' }}

# Run the application for testing purposes
run *args:
    env RUST_BACKTRACE=full cargo run --release {{args}}

# Installs files
install:
    install -Dm0755 {{ cargo-target-dir / 'release' / name }} {{bin-dst}}
    install -Dm0644 {{ 'target' / 'xdgen' / 'app.desktop' }} {{desktop-dst}}
    install -Dm0644 {{ 'target' / 'xdgen' / 'app.metainfo.xml' }} {{appdata-dst}}
    install -Dm0644 resources/icon.svg {{icon-dst}}

# Uninstalls installed files
uninstall:
    rm {{bin-dst}} {{desktop-dst}} {{icon-dst}}

# Vendor dependencies locally
vendor:
    mkdir -p .cargo
    cargo vendor --sync Cargo.toml | head -n -1 > .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml
    echo >> .cargo/config.toml
    rm -rf .cargo vendor

# Extracts vendored dependencies
vendor-extract:
    rm -rf vendor
    tar pxf vendor.tar

# Packs the release binary into an AppImage; needs `appimagetool` on PATH
appimage: build-release
    rm -rf {{cargo-target-dir}}/AppDir
    install -Dm0755 {{cargo-target-dir}}/release/{{name}} {{cargo-target-dir}}/AppDir/usr/bin/{{name}}
    install -Dm0644 target/xdgen/app.desktop {{cargo-target-dir}}/AppDir/{{desktop}}
    install -Dm0644 resources/icon.svg {{cargo-target-dir}}/AppDir/{{appid}}.svg
    install -Dm0644 target/xdgen/app.metainfo.xml {{cargo-target-dir}}/AppDir/usr/share/metainfo/{{appdata}}
    ln -s usr/bin/{{name}} {{cargo-target-dir}}/AppDir/AppRun
    ARCH=x86_64 appimagetool --no-appstream {{cargo-target-dir}}/AppDir {{cargo-target-dir}}/{{name}}-x86_64.AppImage
