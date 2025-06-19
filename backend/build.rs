use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../frontend/src/**");
    println!("cargo:rerun-if-changed=../frontend/package.json");
    println!("cargo:rerun-if-changed=../frontend/svelte.config.js");
    println!("cargo:rerun-if-changed=../frontend/vite.config.ts");

    // Get the project root directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_root = Path::new(&manifest_dir).parent().unwrap();
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

    // Check if frontend directory exists
    if !frontend_dir.exists() {
        println!("cargo:warning=Frontend directory not found, skipping build");
        return;
    }

    // Check if package.json exists
    let package_json = frontend_dir.join("package.json");
    if !package_json.exists() {
        println!("cargo:warning=package.json not found, skipping frontend build");
        return;
    }

    // Build the frontend
    let build_status = Command::new("pnpm")
        .arg("run")
        .arg("build")
        .current_dir(frontend_dir)
        .status();

    match build_status {
        Ok(status) if status.success() => {
            println!("cargo:warning=Frontend built successfully");

            // Check if build output exists
            let build_dir = frontend_dir.join("build");
            if build_dir.exists() {
                println!(
                    "cargo:warning=Frontend build output found at: {}",
                    build_dir.display()
                );
            } else {
                println!("cargo:warning=Frontend build completed but output directory not found");
            }
        }
        Ok(status) => {
            println!(
                "cargo:warning=Frontend build failed (exit code: {})",
                status.code().unwrap_or(-1)
            );
        }
        Err(e) => {
            println!("cargo:warning=Failed to run pnpm build: {}", e);
        }
    }
}

fn create_empty_build_dir(frontend_dir: &Path) {
    use std::fs;

    let build_dir = frontend_dir.join("build");
    if !build_dir.exists() {
        if let Err(e) = fs::create_dir_all(&build_dir) {
            println!(
                "cargo:warning=Failed to create empty build directory: {}",
                e
            );
            return;
        }

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
        <p>To build the frontend, run: <code>BUILD_FRONTEND=1 cargo build</code></p>
    </div>
</body>
</html>"#;

        if let Err(e) = fs::write(&index_html, dev_content) {
            println!("cargo:warning=Failed to create dev index.html: {}", e);
        }
    }
}
