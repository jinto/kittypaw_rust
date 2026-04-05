use std::path::PathBuf;

const PLIST_LABEL: &str = "com.kittypaw.daemon";

fn plist_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home)
        .join("Library/LaunchAgents")
        .join(format!("{PLIST_LABEL}.plist"))
}

fn kittypaw_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".kittypaw")
}

pub(crate) fn run_daemon_install() {
    #[cfg(not(target_os = "macos"))]
    {
        eprintln!("Daemon install is only supported on macOS.");
        return;
    }

    #[cfg(target_os = "macos")]
    {
        let bin_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("kittypaw"));
        let kp_dir = kittypaw_dir();
        std::fs::create_dir_all(&kp_dir).ok();

        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{PLIST_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>serve</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{}/daemon.log</string>
    <key>StandardErrorPath</key>
    <string>{}/daemon.err</string>
</dict>
</plist>"#,
            bin_path.display(),
            kp_dir.display(),
            kp_dir.display(),
        );

        let path = plist_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        match std::fs::write(&path, &plist) {
            Ok(()) => {
                println!("Plist written to {}", path.display());
                let status = std::process::Command::new("launchctl")
                    .args(["load", "-w"])
                    .arg(&path)
                    .status();
                match status {
                    Ok(s) if s.success() => println!("Daemon installed and started."),
                    Ok(s) => eprintln!("launchctl load exited with: {s}"),
                    Err(e) => eprintln!("Failed to run launchctl: {e}"),
                }
            }
            Err(e) => eprintln!("Failed to write plist: {e}"),
        }
    }
}

pub(crate) fn run_daemon_uninstall() {
    #[cfg(not(target_os = "macos"))]
    {
        eprintln!("Daemon uninstall is only supported on macOS.");
        return;
    }

    #[cfg(target_os = "macos")]
    {
        let path = plist_path();
        if path.exists() {
            let _ = std::process::Command::new("launchctl")
                .args(["unload"])
                .arg(&path)
                .status();
            match std::fs::remove_file(&path) {
                Ok(()) => println!("Daemon uninstalled."),
                Err(e) => eprintln!("Failed to remove plist: {e}"),
            }
        } else {
            println!("Daemon is not installed.");
        }
    }
}

pub(crate) fn run_daemon_status() {
    let path = plist_path();
    if !path.exists() {
        println!("Daemon: not installed");
        return;
    }
    println!("Plist: {}", path.display());

    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("launchctl")
            .args(["list"])
            .output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.contains(PLIST_LABEL) {
                    println!("Status: running");
                } else {
                    println!("Status: installed but not running");
                }
            }
            Err(_) => println!("Status: unknown (launchctl failed)"),
        }
    }

    #[cfg(not(target_os = "macos"))]
    println!("Status: not supported on this platform");
}
