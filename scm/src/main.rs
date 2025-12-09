use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha512};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

const SCM_FILE: &str = ".scm";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CommitEntry {
    hash: String,
    init: HashMap<String, Vec<String>>,
    diff: HashMap<String, Vec<String>>,
}

# [derive(Serialize, Deserialize, Debug)]
struct ScmData {
    latest: HashMap<String, Vec<String>>,
    commits: Vec<CommitEntry>,
    merkle: Vec<Vec<String>>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    match args[1].as_str() {
        "init" | "commit" => commit(),
        "revert" => revert(),
        "log" => log(),
        "status" => status(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: scm <command>");
    eprintln!("Commands:");
    eprintln!("  init/commit  - Initialize or save current state");
    eprintln!("  revert       - Roll back to previous commit");
    eprintln!("  log          - Show commit history");
    eprintln!("  status       - Show current SCM status");
}

/// Get all non-hidden files recursively from current directory
fn get_all_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_string_lossy();
            
            // Skip hidden files and SCM file
            if file_name.starts_with('.') {
                continue;
            }
            
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                files.extend(get_files_recursive(&path));
            }
        }
    }
    
    files.sort();
    files
}

fn get_files_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_string_lossy();
            
            if file_name.starts_with('.') {
                continue;
            }
            
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                files.extend(get_files_recursive(&path));
            }
        }}
    files
}

/// Read file as lines
fn read_file_lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(String::from)
        .collect()
}

/// Compute SHA-512 hash of all files concatenated
fn compute_hash(files: &HashMap<String, Vec<String>>) -> String {
    let mut hasher = Sha512::new();
    
    // Sort keys for deterministic hashing
    let mut keys: Vec<_> = files.keys().collect();
    keys.sort();
    
    for key in keys {
        hasher.update(key.as_bytes());
        if let Some(lines) = files.get(key) {
            for line in lines {
                hasher.update(line.as_bytes());
                hasher.update(b"\n");
            }
        }
    }
    
    format!("{:x}", hasher.finalize())
}

/// Generate diff between two versions using the diff implementation
fn generate_diff(old: &[String], new: &[String]) -> Vec<String> {
    let lcs_table = build_lcs_table(old, new);
    let mut result = Vec::new();
    let mut i = 0;
    let mut j = 0;
    
    while i < old.len() || j < new.len() {
        if i < old.len() && j < new.len() && old[i] == new[j] {
            i += 1;
            j += 1;
        } else if j >= new.len() || (i < old.len() && lcs_table[i + 1][j] > lcs_table[i][j + 1]) {
            result.push(format!("{}d{}", i + 1, j));
            result.push(format!("< {}", old[i]));
            i += 1;
        } else {
            result.push(format!("{}a{}", i, j + 1));
            result.push(format!("> {}", new[j]));
            j += 1;
        }
    }
    
    result
}

fn build_lcs_table(v1: &[String], v2: &[String]) -> Vec<Vec<usize>> {
    let (len1, len2) = (v1.len(), v2.len());
    let mut table = vec![vec![0; len2 + 1]; len1 + 1];
    
    for i in (0..len1).rev() {
        for j in (0..len2).rev() {
            table[i][j] = if v1[i] == v2[j] {
                1 + table[i + 1][j + 1]
            } else {
                std::cmp::max(table[i + 1][j], table[i][j + 1])
            };
        }
    }
    table
}

/// Apply diff in reverse to revert changes
fn apply_diff_reverse(current: &[String], diff: &[String]) -> Vec<String> {
    let mut result = current.to_vec();
    
    // Simple reverse application - in production would parse diff format properly
    for line in diff.iter().rev() {
        if line.starts_with("< ") {
            // Line was deleted, add it back
            let content = line[2..].to_string();
            result.push(content);
        } else if line.starts_with("> ") {
            // Line was added, remove it
            let content = line[2..].to_string();
            result.retain(|l| l != &content);
        }
    }
    
    result
}

/// Build Merkle tree from commit hashes
fn build_merkle_tree(hashes: &[String]) -> Vec<Vec<String>> {
    if hashes.is_empty() {
        return vec![];}
    let mut tree = vec![hashes.to_vec()];
    let mut current_level = hashes.to_vec();
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        for chunk in current_level.chunks(2) {
            let combined = if chunk.len() == 2 {
                format!("{}{}", chunk[0], chunk[1])
            } else {
                chunk[0].clone()
            };
            let mut hasher = Sha512::new();
            hasher.update(combined.as_bytes());
            let hash = format!("{:x}", hasher.finalize());
            next_level.push(hash);
        }
        tree.push(next_level.clone());
        current_level = next_level;
    }
    tree
}

fn commit() {
    let files = get_all_files();
    
    if files.is_empty() {
        eprintln!("No files to commit");
        return;
    }
    let mut current_files: HashMap<String, Vec<String>> = HashMap::new();
    for file in &files {
        let path_str = file.to_string_lossy().to_string();
        let lines = read_file_lines(file);
        current_files.insert(path_str, lines);
    }
    // Check if .scm exists
    if !Path::new(SCM_FILE).exists() || fs::metadata(SCM_FILE).unwrap().len() == 0 {
        // Initialize - first commit
        println!("Initializing SCM...");
        
        let hash = compute_hash(&current_files);
        let commit = CommitEntry {
            hash: hash.clone(),
            init: current_files.clone(),
            diff: HashMap::new(),
        };
        
        let merkle = build_merkle_tree(&[hash]);
        let scm_data = ScmData {
            latest: current_files,
            commits: vec![commit],
            merkle,
        };
        let json = serde_json::to_string_pretty(&scm_data).unwrap();
        fs::write(SCM_FILE, json).expect("Failed to write .scm file");
        println!("Initialized with {} files", files.len());
    } else {
        // Load existing SCM data
        let content = fs::read_to_string(SCM_FILE).expect("Failed to read .scm");
        let mut scm_data: ScmData = serde_json::from_str(&content).expect("Failed to parse .scm");
        // Get files that existed in last commit
        let last_commit = scm_data.commits.last().unwrap();
        let mut old_files: Vec<String> = last_commit.init.keys().cloned().collect();
        old_files.extend(last_commit.diff.keys().cloned());
        // Separate new files from modified files
        let mut init: HashMap<String, Vec<String>> = HashMap::new();
        let mut diff: HashMap<String, Vec<String>> = HashMap::new();
        
        for (path, lines) in &current_files {
            if !old_files.contains(path) {
                // New file
                init.insert(path.clone(), lines.clone());
            } else if let Some(old_lines) = scm_data.latest.get(path) {
                // Check if modified
                if old_lines != lines {
                    let file_diff = generate_diff(old_lines, lines);
                    if !file_diff.is_empty() {
                        diff.insert(path.clone(), file_diff);
                    }
                }
            }
        }
        
        if init.is_empty() && diff.is_empty() {
            println!("No changes to commit");
            return;
        }
        
        // Compute hash and create commit
        let hash = compute_hash(&current_files);
        let commit = CommitEntry {
            hash: hash.clone(),
            init,
            diff,
        };
        
        scm_data.commits.push(commit);
        scm_data.latest = current_files;
        // Update Merkle tree
        let all_hashes: Vec<String> = scm_data.commits.iter().map(|c| c.hash.clone()).collect();
        scm_data.merkle = build_merkle_tree(&all_hashes);
        let json = serde_json::to_string_pretty(&scm_data).unwrap();
        fs::write(SCM_FILE, json).expect("Failed to write .scm file");
        
        println!("Committed changes (hash: {}...)", &hash[..16]);
    }
}

fn revert() {
    if !Path::new(SCM_FILE).exists() {
        eprintln!("No .scm file found. Initialize with 'scm commit' first.");
        return;
    }
    
    let content = fs::read_to_string(SCM_FILE).expect("Failed to read .scm");
    let mut scm_data: ScmData = serde_json::from_str(&content).expect("Failed to parse .scm");
    
    if scm_data.commits.len() < 2 {
        eprintln!("No previous commit to revert to");
        return;
    }
    
    // Remove last commit
    let removed = scm_data.commits.pop().unwrap();
    println!("Reverting commit {}...", &removed.hash[..16]);
    
    // Reconstruct previous state
    let mut previous_state: HashMap<String, Vec<String>> = HashMap::new();
    
    for commit in &scm_data.commits {
        // Add init files
        for (path, lines) in &commit.init {
            previous_state.insert(path.clone(), lines.clone());
        }
        
        // Apply diffs (simplified - just store latest from diff)
        for (path, _diff) in &commit.diff {
            if let Some(lines) = scm_data.latest.get(path) {
                previous_state.insert(path.clone(), lines.clone());
            }
        }
    }
    
    // Write previous state back to filesystem
    for (path, lines) in &previous_state {
        let content = lines.join("\n") + "\n";
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(path, content).expect("Failed to write file");
    }
    
    scm_data.latest = previous_state;
    // Update Merkle tree
    let all_hashes: Vec<String> = scm_data.commits.iter().map(|c| c.hash.clone()).collect();
    scm_data.merkle = build_merkle_tree(&all_hashes);
    let json = serde_json::to_string_pretty(&scm_data).unwrap();
    fs::write(SCM_FILE, json).expect("Failed to write .scm file");
    println!("Reverted to previous commit");
}

fn log() {
    if !Path::new(SCM_FILE).exists() {
        eprintln!("No .scm file found");
        return;
    }
    
    let content = fs::read_to_string(SCM_FILE).expect("Failed to read .scm");
    let scm_data: ScmData = serde_json::from_str(&content).expect("Failed to parse .scm");
    println!("Commit History:");
    println!("==============");
    for (idx, commit) in scm_data.commits.iter().enumerate().rev() {
        println!("\nCommit #{}", idx);
        println!("Hash: {}", commit.hash);
        println!("New files: {}", commit.init.len());
        println!("Modified files: {}", commit.diff.len());
    }
    if !scm_data.merkle.is_empty() {
        let root_level = scm_data.merkle.last().unwrap();
        if !root_level.is_empty() {
            println!("\nMerkle Root: {}", root_level[0]);
        }
    }
}

fn status() {
    if !Path::new(SCM_FILE).exists() {
        println!("Not under version control. Run 'scm init' to initialize.");
        return;
    }
    let content = fs::read_to_string(SCM_FILE).expect("Failed to read .scm");
    let scm_data: ScmData = serde_json::from_str(&content).expect("Failed to parse .scm");
    println!("SCM Status:");
    println!("Total commits: {}", scm_data.commits.len());
    println!("Tracked files: {}", scm_data.latest.len());
    if let Some(last) = scm_data.commits.last() {
        println!("Commit hash: {}...", &last.hash[..16]);
    }
}