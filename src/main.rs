use anyhow::Result;
use clap::Parser;
use std::env;
use std::fs;
use std::path::Path;
use workspace_manager::*;

fn main() -> Result<()> {
    let args = Args::parse();
    let current_dir = env::current_dir()?;
    let base_path = Path::new(&args.path).canonicalize()?;

    let workspace_name = args.name.clone().unwrap_or_else(|| {
        current_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });

    let workspace_filename = format!("{}.code-workspace", workspace_name);
    let workspace = create_workspace(
        &base_path,
        &workspace_name,
        args.exclude_current,
        args.update_task,
        &args,
    )?;

    let workspace_json = serde_json::to_string_pretty(&workspace)?;
    fs::write(&workspace_filename, workspace_json)?;

    println!(
        "Workspace file '{}' updated successfully!",
        workspace_filename
    );
    Ok(())
}
