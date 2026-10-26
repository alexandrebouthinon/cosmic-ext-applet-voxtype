// SPDX-License-Identifier: MPL-2.0

use std::{
    env, fs, io,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

const BINARY_NAME: &str = "cosmic-ext-applet-voxtype";
const APP_ID: &str = "io.github.alexandrebouthinon.CosmicExtAppletVoxtype";
const DESKTOP: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/xdgen/app.desktop"
));
const ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/icon.svg"));

struct InstallPaths {
    binary: PathBuf,
    desktop: PathBuf,
    icon: PathBuf,
}

fn paths(home: &Path) -> InstallPaths {
    let local = home.join(".local");
    InstallPaths {
        binary: local.join("bin").join(BINARY_NAME),
        desktop: local
            .join("share/applications")
            .join(format!("{APP_ID}.desktop")),
        icon: local
            .join("share/icons/hicolor/scalable/apps")
            .join(format!("{APP_ID}.svg")),
    }
}

fn rewrite_exec(desktop: &str, binary: &Path) -> String {
    let mut result = desktop
        .lines()
        .map(|line| {
            if line.starts_with("Exec=") {
                format!("Exec={}", binary.display())
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if desktop.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn home() -> io::Result<PathBuf> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    if !home.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "HOME must be an absolute path",
        ));
    }
    Ok(home)
}

fn write_file(path: &Path, contents: &[u8]) -> io::Result<()> {
    fs::create_dir_all(path.parent().expect("installation file has a parent"))?;
    fs::write(path, contents)?;
    println!("{}", path.display());
    Ok(())
}

fn install_in(home: &Path, executable: &Path) -> io::Result<()> {
    let paths = paths(home);
    fs::create_dir_all(paths.binary.parent().expect("binary has a parent"))?;
    let already_installed =
        paths.binary.exists() && fs::canonicalize(executable)? == fs::canonicalize(&paths.binary)?;
    if already_installed {
        fs::set_permissions(&paths.binary, fs::Permissions::from_mode(0o755))?;
    } else {
        // Rename over the target: writing into it fails with ETXTBSY while the panel runs it.
        let staged = paths.binary.with_extension("new");
        fs::copy(executable, &staged)?;
        fs::set_permissions(&staged, fs::Permissions::from_mode(0o755))?;
        fs::rename(&staged, &paths.binary)?;
    }
    println!("{}", paths.binary.display());

    write_file(
        &paths.desktop,
        rewrite_exec(DESKTOP, &paths.binary).as_bytes(),
    )?;
    write_file(&paths.icon, ICON)
}

pub fn install() -> io::Result<()> {
    // Inside an AppImage, `current_exe()` is in a mount that vanishes on exit.
    let source = env::var_os("APPIMAGE").map_or_else(env::current_exe, |path| Ok(path.into()))?;
    install_in(&home()?, &source)
}

fn uninstall_from(home: &Path) -> io::Result<()> {
    let paths = paths(home);
    for path in [&paths.binary, &paths.desktop, &paths.icon] {
        match fs::remove_file(path) {
            Ok(()) => println!("{}", path.display()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

pub fn uninstall() -> io::Result<()> {
    uninstall_from(&home()?)
}

#[cfg(test)]
mod tests {
    use super::{ICON, install_in, paths, rewrite_exec, uninstall_from};
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn rewrites_only_exec_line() {
        let binary = PathBuf::from("/tmp/example/bin/cosmic-ext-applet-voxtype");
        let desktop =
            "[Desktop Entry]\nExec=old-command %F\nTryExec=old-command\nX-CosmicApplet=true\n";
        assert_eq!(
            rewrite_exec(desktop, &binary),
            format!(
                "[Desktop Entry]\nExec={}\nTryExec=old-command\nX-CosmicApplet=true\n",
                binary.display()
            )
        );
    }

    #[test]
    fn installs_and_uninstalls_in_temporary_home() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time predates Unix epoch")
            .as_nanos();
        let home =
            std::env::temp_dir().join(format!("voxtype-install-{}-{unique}", std::process::id()));
        let source = home.join("source-binary");
        fs::create_dir(&home).unwrap();
        fs::write(&source, b"test binary").unwrap();

        let installed = paths(&home);
        install_in(&home, &source).unwrap();
        assert_eq!(fs::read(&installed.binary).unwrap(), b"test binary");
        assert_eq!(
            fs::metadata(&installed.binary)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o755
        );
        assert_eq!(fs::read(&installed.icon).unwrap(), ICON);
        let desktop = fs::read_to_string(&installed.desktop).unwrap();
        assert!(
            desktop
                .lines()
                .any(|line| line == format!("Exec={}", installed.binary.display()))
        );
        assert!(desktop.contains("X-CosmicApplet=true"));

        fs::write(&source, b"replacement binary").unwrap();
        install_in(&home, &source).unwrap();
        assert_eq!(fs::read(&installed.binary).unwrap(), b"replacement binary");
        install_in(&home, &installed.binary).unwrap();
        assert_eq!(fs::read(&installed.binary).unwrap(), b"replacement binary");

        uninstall_from(&home).unwrap();
        uninstall_from(&home).unwrap();
        assert!(!installed.binary.exists());
        assert!(!installed.desktop.exists());
        assert!(!installed.icon.exists());
        fs::remove_dir_all(&home).unwrap();
    }
}
