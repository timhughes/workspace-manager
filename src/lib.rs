use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "VS Code workspace manager that creates workspace entries for folders"
)]
pub struct Args {
    /// Path to scan for workspace folders
    #[arg(
        short,
        long,
        default_value = ".",
        help = "Directory to scan for workspace folders"
    )]
    pub path: String,

    /// Exclude current directory from workspace
    #[arg(
        short,
        long,
        help = "Exclude current directory from workspace (default: include)"
    )]
    pub exclude_current: bool,

    /// Name for the workspace file (without .code-workspace extension)
    #[arg(short, long, help = "Custom name for the workspace file")]
    pub name: Option<String>,

    /// Force update of workspace tasks
    #[arg(short, long, help = "Update workspace task even if file exists")]
    pub update_task: bool,
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
    // Add a catch-all field for other sections
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
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

pub fn create_workspace_folder(path: &Path, base_path: &Path, scan_path: &Path) -> Result<WorkspaceFolder> {
    let name = path.file_name()
        .context("Invalid folder name")?
        .to_string_lossy()
        .into_owned();

    let relative_path = if path == scan_path {
        scan_path.strip_prefix(base_path)
            .unwrap_or(Path::new("."))
            .to_string_lossy()
            .into_owned()
    } else {
        pathdiff::diff_paths(path, base_path)
            .context("Failed to calculate relative path")?
            .to_string_lossy()
            .into_owned()
    };

    Ok(WorkspaceFolder {
        path: relative_path,
        name: format!("📦 {}", name),
    })
}

pub fn merge_tasks(existing: Option<Tasks>, new_task: Task) -> Tasks {
    let mut tasks = existing.unwrap_or_else(|| Tasks {
        version: "2.0.0".to_string(),
        tasks: Vec::new(),
    });

    // Remove any existing update workspace task
    tasks.tasks.retain(|task| task.label != "Update Workspace");

    // Add the new update workspace task
    tasks.tasks.push(new_task);

    tasks
}

fn args_to_vec(args: &Args) -> Vec<String> {
    let mut task_args = vec![];
    
    if let Some(name) = &args.name {
        task_args.extend_from_slice(&["--name".to_string(), name.clone()]);
    }
    if args.exclude_current {
        task_args.push("--exclude-current".to_string());
    }
    task_args.extend_from_slice(&["--path".to_string(), args.path.clone()]);
    
    task_args
}

pub fn create_workspace_task(args: &Args) -> Tasks {
    Tasks {
        version: "2.0.0".to_string(),
        tasks: vec![Task {
            label: "Update Workspace".to_string(),
            task_type: "process".to_string(),
            command: env::current_exe()
                .unwrap_or_else(|_| PathBuf::from("workspace-manager"))
                .to_string_lossy()
                .to_string(),
            args: args_to_vec(args),
        }],
    }
}

pub fn create_workspace(
    scan_path: &Path,
    workspace_name: &str,
    exclude_current: bool,
    update_task: bool,
    args: &Args,
) -> Result<WorkspaceFile> {
    let base_path = env::current_dir()?;
    let mut workspace = WorkspaceFile::default();
    let workspace_file = format!("{}.code-workspace", workspace_name);
    
    // Read existing workspace file if it exists
    if Path::new(&workspace_file).exists() {
        if let Ok(content) = fs::read_to_string(&workspace_file) {
            if let Ok(existing_workspace) = serde_json::from_str::<WorkspaceFile>(&content) {
                // Preserve other sections
                workspace.other = existing_workspace.other;
                // Preserve existing tasks
                workspace.tasks = existing_workspace.tasks;
                if update_task {
                    // Only update our specific task
                    if let Some(tasks) = &mut workspace.tasks {
                        tasks.tasks.retain(|t| t.label != "Update Workspace");
                        tasks.tasks.push(Task {
                            label: "Update Workspace".to_string(),
                            task_type: "process".to_string(),
                            command: env::current_exe()?.to_string_lossy().to_string(),
                            args: args_to_vec(args),
                        });
                    } else {
                        workspace.tasks = Some(create_workspace_task(args));
                    }
                }
            }
        }
    } else {
        workspace.tasks = Some(create_workspace_task(args));
    }

    // Update folders
    if !exclude_current {
        workspace.folders.push(WorkspaceFolder {
            path: ".".to_string(),
            name: format!("🏗️ {}", workspace_name),
        });
    }

    let dirs = scan_directories(scan_path)?;
    for dir in dirs {
        let folder = create_workspace_folder(&dir, &base_path, scan_path)?;
        workspace.folders.push(folder);
    }

    Ok(workspace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_workspace() -> Result<()> {
        let temp = TempDir::new()?;
        let base_path = env::current_dir()?;
        let scan_path = temp.path();
        
        // Create test directories
        fs::create_dir_all(scan_path.join("folder1"))?;
        fs::create_dir_all(scan_path.join("folder2"))?;

        let args = Args {
            path: scan_path.to_string_lossy().to_string(),
            exclude_current: false,
            name: Some("test".to_string()),
            update_task: true,
        };

        let workspace = create_workspace(
            scan_path,
            "test",
            args.exclude_current,
            args.update_task,
            &args,
        )?;

        // Verify structure
        assert_eq!(workspace.folders.len(), 3);
        
        // Current directory should be first
        assert_eq!(workspace.folders[0].path, ".");
        assert_eq!(workspace.folders[0].name, "🏗️ test");

        // Get relative paths for comparison
        let rel_path = pathdiff::diff_paths(&scan_path, &base_path)
            .expect("Failed to get relative path");
        
        // Verify folder paths are relative to workspace file location
        let expected_path1 = rel_path.join("folder1");
        let expected_path2 = rel_path.join("folder2");

        assert!(workspace.folders.iter().any(|f| 
            f.path == expected_path1.to_string_lossy() && f.name == "📦 folder1"
        ), "folder1 not found with correct path");
        
        assert!(workspace.folders.iter().any(|f| 
            f.path == expected_path2.to_string_lossy() && f.name == "📦 folder2"
        ), "folder2 not found with correct path");

        Ok(())
    }

    #[test]
    fn test_create_workspace_folder() -> Result<()> {
        let temp = TempDir::new()?;
        let base_path = env::current_dir()?;
        let scan_path = temp.path();
        
        let test_dir = scan_path.join("nested").join("test_folder");
        fs::create_dir_all(&test_dir)?;
        
        let folder = create_workspace_folder(&test_dir, &base_path, scan_path)?;
        
        let expected_path = pathdiff::diff_paths(&test_dir, &base_path)
            .expect("Failed to get relative path");
            
        assert_eq!(folder.path, expected_path.to_string_lossy());
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
    fn test_merge_tasks() {
        let existing = Tasks {
            version: "2.0.0".to_string(),
            tasks: vec![Task {
                label: "Existing Task".to_string(),
                task_type: "shell".to_string(),
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
            }],
        };

        let new_task = Task {
            label: "Update Workspace".to_string(),
            task_type: "process".to_string(),
            command: "workspace-manager".to_string(),
            args: vec![],
        };

        let merged = merge_tasks(Some(existing), new_task);

        assert_eq!(merged.tasks.len(), 2);
        assert!(merged.tasks.iter().any(|t| t.label == "Existing Task"));
        assert!(merged.tasks.iter().any(|t| t.label == "Update Workspace"));
    }
}
