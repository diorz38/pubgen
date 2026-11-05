mod actions;
mod audit;
mod branding;
mod brs;
mod control_client;
mod covers;
pub mod embedded_assets;
mod glossary;
mod header_footer;
mod i18n;
mod ingest;
mod main_ui;
mod metadata_form;
mod process;
mod profiles;
mod project;
mod slots;
pub mod telemetry;
mod toc;
mod translate;
mod ui;
mod vector;

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--smoke") {
        return smoke();
    }
    ui::run()
}

fn smoke() -> Result<()> {
    let port: u16 = std::env::var("PUBGEN_SMOKE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(crate::main_ui::DC_PORT);

    let exe = process::resolve_exe()
        .context("no DesignCraft executable found (set PUBGEN_DESIGNCRAFT_EXE)")?;
    println!("designcraft exe: {}", exe.display());
    match process::resolve_cli() {
        Some(cli) => println!("designcraft-cli: {}", cli.display()),
        None => println!("designcraft-cli: not found (batch fill will be unavailable)"),
    }

    let mut proc = process::spawn(&exe, port)?;
    println!("designcraft spawned on port {port}");

    let mut client = None;
    for _ in 0..40 {
        if let Ok(c) = control_client::ControlClient::connect(("127.0.0.1", port)) {
            client = Some(c);
            break;
        }
        if process::poll(&mut proc).is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    let mut client =
        client.context("designcraft did not accept a control connection in time")?;
    println!("control connection established");

    let result = client.call("engine.commands", serde_json::json!({}))?;
    let count = result
        .as_array()
        .map(|a| a.len())
        .ok_or_else(|| anyhow::anyhow!("engine.commands result is not a command list"))?;
    println!("engine.commands ok, {count} commands registered");

    process::kill(&mut proc);
    if count == 0 {
        anyhow::bail!("engine.commands returned an empty command list");
    }
    println!("SMOKE PASS");
    Ok(())
}
