use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../frontend/src");
    println!("cargo:rerun-if-changed=../frontend/package.json");
    println!("cargo:rerun-if-changed=../frontend/svelte.config.js");
    println!("cargo:rerun-if-changed=../frontend/vite.config.ts");

    // Get the project root directory
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo");
    let project_root = Path::new(&manifest_dir)
        .parent()
        .expect("CARGO_MANIFEST_DIR must have a parent directory");
    let frontend_dir = project_root.join("frontend");

    // Check if BUILD_FRONTEND is set
    let no_build_frontend = env::var("NO_BUILD_FRONTEND").is_ok();

    if no_build_frontend {
        println!(
            "cargo:warning=Skipping frontend build in debug mode. Unset NO_BUILD_FRONTEND=1 to build."
        );
        // Create empty build directory to prevent missing file errors
        create_empty_build_dir(&frontend_dir);
    } else {
        build_frontend(&frontend_dir);
    }
}

fn build_frontend(frontend_dir: &Path) {
    println!(
        "cargo:warning=Building frontend at: {}",
        frontend_dir.display()
    );

    if !frontend_dir.exists() {
        panic!(
            "Frontend directory not found at: {}",
            frontend_dir.display()
        );
    }

    let package_json = frontend_dir.join("package.json");
    if !package_json.exists() {
        panic!(
            "package.json not found at: {}",
            package_json.display()
        );
    }

    let build_status = Command::new("pnpm")
        .arg("run")
        .arg("build")
        .current_dir(frontend_dir)
        .status();

    match build_status {
        Ok(status) if status.success() => {
            let build_dir = frontend_dir.join("build");
            if !build_dir.exists() {
                panic!("Frontend build completed but output directory not found");
            }
            println!(
                "cargo:warning=Frontend built successfully at: {}",
                build_dir.display()
            );
        }
        Ok(status) => {
            panic!(
                "Frontend build failed with exit code: {}",
                status.code().unwrap_or(-1)
            );
        }
        Err(e) => {
            panic!("Failed to run pnpm build: {}", e);
        }
    }
}

fn create_empty_build_dir(frontend_dir: &Path) {
    use std::fs;

    let build_dir = frontend_dir.join("build");
    if !build_dir.exists() {
        fs::create_dir_all(&build_dir).unwrap_or_else(|e| {
            panic!(
                "Failed to create empty build directory at {}: {}",
                build_dir.display(),
                e
            );
        });

        // Create a simple index.html for development
        let index_html = build_dir.join("index.html");
        let dev_content = r#"<!DOCTYPE html>
<html>
<head>
    <title>Tabletop Atlas - Development Mode</title>
    <style>
        body { font-family: sans-serif; text-align: center; margin-top: 100px; }
        .dev-notice { color: #666; background: #f0f0f0; padding: 20px; border-radius: 8px; max-width: 500px; margin: 0 auto; }
    </style>
</head>
<body>
    <div class="dev-notice">
        <h1>Development Mode</h1>
        <p>The frontend is running in development mode.</p>
        <p>API endpoints are available at <code>/api/*</code></p>
        <p>To build the frontend, unset <code>NO_BUILD_FRONTEND</code> and run: <code>cargo build</code></p>
    </div>
</body>
</html>"#;

        fs::write(&index_html, dev_content).unwrap_or_else(|e| {
            panic!(
                "Failed to create dev index.html at {}: {}",
                index_html.display(),
                e
            );
        });
    }
}
