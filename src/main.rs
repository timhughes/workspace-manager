use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use serde::{Serialize, Deserialize};
use clap::Parser;
use anyhow::{Result, Context};

#[derive(Parser, Debug)]
#[command(author, version, about = "VS Code workspace manager that creates workspace entries for folders")]
struct Args {
    /// Path to scan for workspace folders
    #[arg(short, long, default_value = ".", help = "Directory to scan for workspace folders")]
    path: String,

    /// Include current directory as first workspace folder
    #[arg(short, long, help = "Add current directory as first workspace folder with 🏗️ prefix")]
    include_current: bool,

    /// Name for the workspace file (without .code-workspace extension)
    #[arg(short, long, help = "Custom name for the workspace file")]
    name: Option<String>,

    /// Force update of workspace tasks
    #[arg(short, long, help = "Update workspace tasks even if file exists")]
    update_tasks: bool,
}

#[derive(Serialize, Deserialize)]
struct WorkspaceFolder {
    path: String,
    name: String,
}

#[derive(Serialize, Deserialize, Default)]
struct TaskDefinition {
    #[serde(rename = "type")]
    task_type: String,
    command: String,
    args: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
struct Task {
    label: String,
    #[serde(rename = "type")]
    task_type: String,
    command: String,
    args: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
struct Tasks {
    version: String,
    tasks: Vec<Task>,
}

#[derive(Serialize, Deserialize, Default)]
struct WorkspaceFile {
    folders: Vec<WorkspaceFolder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tasks: Option<Tasks>,
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

fn scan_directories(base_path: &Path) -> Result<Vec<PathBuf>> {
    let mut dirs = vec![];
    for entry in fs::read_dir(base_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && !is_hidden(&path) {
            dirs.push(path);
        }
    }
    Ok(dirs)
}

fn create_workspace_folder(path: &Path, base_path: &Path) -> Result<WorkspaceFolder> {
    let relative_path = path.strip_prefix(base_path)?;
    let name = path.file_name()
        .context("Invalid folder name")?
        .to_string_lossy()
        .into_owned();

    Ok(WorkspaceFolder {
        path: relative_path.to_string_lossy().to_string(),
        name: format!("📦 {}", name),
    })
}

fn create_workspace_task(args: &Args) -> Tasks {
    let current_exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("workspace-manager"));
    let mut task_args = vec![];
    
    if let Some(name) = &args.name {
        task_args.extend_from_slice(&["--name".to_string(), name.clone()]);
    }
    if args.include_current {
        task_args.push("--include-current".to_string());
    }
    task_args.extend_from_slice(&["--path".to_string(), args.path.clone()]);

    Tasks {
        version: "2.0.0".to_string(),
        tasks: vec![Task {
            label: "Update Workspace".to_string(),
            task_type: "process".to_string(),
            command: current_exe.to_string_lossy().to_string(),
            args: task_args,
        }],
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let base_path = Path::new(&args.path).canonicalize()?;
    
    let workspace_name = args.name.clone().unwrap_or_else(|| {
        base_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let workspace_filename = format!("{}.code-workspace", workspace_name);

    let mut workspace = if Path::new(&workspace_filename).exists() {
        let content = fs::read_to_string(&workspace_filename)?;
        let mut existing: WorkspaceFile = serde_json::from_str(&content).unwrap_or_default();
        if args.update_tasks {
            existing.tasks = Some(create_workspace_task(&args));
        }
        existing
    } else {
        let mut new_workspace = WorkspaceFile::default();
        new_workspace.tasks = Some(create_workspace_task(&args));
        new_workspace
    };

    workspace.folders.clear();

    if args.include_current {
        workspace.folders.push(WorkspaceFolder {
            path: ".".to_string(),
            name: format!("🏗️ {}", workspace_name),
        });
    }

    let dirs = scan_directories(&base_path)?;
    for dir in dirs {
        if !is_hidden(&dir) {
            let folder = create_workspace_folder(&dir, &base_path)?;
            workspace.folders.push(folder);
        }
    }

    let workspace_json = serde_json::to_string_pretty(&workspace)?;
    fs::write(&workspace_filename, workspace_json)?;

    println!("Workspace file '{}' updated successfully!", workspace_filename);
    Ok(())
}