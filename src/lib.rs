use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use serde::{Serialize, Deserialize};
use clap::Parser;
use anyhow::{Result, Context};

#[derive(Parser, Debug)]
#[command(author, version, about = "VS Code workspace manager that creates workspace entries for folders")]
pub struct Args {
    /// Path to scan for workspace folders
    #[arg(short, long, default_value = ".", help = "Directory to scan for workspace folders")]
    pub path: String,

    /// Include current directory as first workspace folder
    #[arg(short, long, help = "Add current directory as first workspace folder with 🏗️ prefix")]
    pub include_current: bool,

    /// Name for the workspace file (without .code-workspace extension)
    #[arg(short, long, help = "Custom name for the workspace file")]
    pub name: Option<String>,

    /// Force update of workspace tasks
    #[arg(short, long, help = "Update workspace tasks even if file exists")]
    pub update_tasks: bool,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct Task {
    pub label: String,
    #[serde(rename = "type")]
    pub task_type: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct Tasks {
    pub version: String,
    pub tasks: Vec<Task>,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct WorkspaceFolder {
    pub path: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct WorkspaceFile {
    pub folders: Vec<WorkspaceFolder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<Tasks>,
}

pub fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

pub fn scan_directories(base_path: &Path) -> Result<Vec<PathBuf>> {
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

pub fn create_workspace_folder(path: &Path, base_path: &Path) -> Result<WorkspaceFolder> {
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

pub fn create_workspace_task(args: &Args) -> Tasks {
    let current_exe = env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("workspace-manager"));
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

pub fn create_workspace(
    scan_path: &Path,
    workspace_name: &str,
    include_current: bool,
    update_tasks: bool,
    args: &Args,
) -> Result<WorkspaceFile> {
    let mut workspace = WorkspaceFile::default();
    
    if include_current {
        workspace.folders.push(WorkspaceFolder {
            path: ".".to_string(),
            name: format!("🏗️ {}", workspace_name),
        });
    }

    let dirs = scan_directories(scan_path)?;
    for dir in dirs {
        if !is_hidden(&dir) {
            let folder = create_workspace_folder(&dir, scan_path)?;
            workspace.folders.push(folder);
        }
    }

    if update_tasks {
        workspace.tasks = Some(create_workspace_task(args));
    }

    Ok(workspace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_create_workspace_folder() -> Result<()> {
        let temp = TempDir::new()?;
        let base = temp.path();
        let test_dir = base.join("test_folder");
        fs::create_dir(&test_dir)?;

        let folder = create_workspace_folder(&test_dir, base)?;
        
        assert_eq!(folder.path, "test_folder");
        assert_eq!(folder.name, "📦 test_folder");
        Ok(())
    }

    #[test]
    fn test_scan_directories() -> Result<()> {
        let temp = TempDir::new()?;
        let base = temp.path();
        
        fs::create_dir(base.join("folder1"))?;
        fs::create_dir(base.join("folder2"))?;
        fs::create_dir(base.join(".hidden"))?;

        let dirs = scan_directories(base)?;
        
        assert_eq!(dirs.len(), 2);
        assert!(dirs.iter().any(|p| p.ends_with("folder1")));
        assert!(dirs.iter().any(|p| p.ends_with("folder2")));
        Ok(())
    }

    #[test]
    fn test_create_workspace() -> Result<()> {
        let temp = TempDir::new()?;
        let base = temp.path();
        
        fs::create_dir(base.join("folder1"))?;
        fs::create_dir(base.join("folder2"))?;

        let args = Args {
            path: base.to_string_lossy().to_string(),
            include_current: true,
            name: Some("test".to_string()),
            update_tasks: true,
        };

        let workspace = create_workspace(
            base,
            "test",
            true,
            true,
            &args,
        )?;

        assert_eq!(workspace.folders.len(), 3); // current + 2 folders
        assert!(workspace.tasks.is_some());
        Ok(())
    }
}