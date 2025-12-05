use std::cmp::max;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <file1> <file2>", args[0]);
        std::process::exit(1);
    }
    
    let v1 = read_file(&args[1]);
    let v2 = read_file(&args[2]);
    print!("{}", diff(&v1, &v2));
}


fn diff(v1: &[String], v2: &[String]) -> String {
    let lcs_table = build_lcs_table(v1, v2);
    generate_diff(v1, v2, &lcs_table)
}

// Separate LCS table buildng for clarity and potential reuse
fn build_lcs_table(v1: &[String], v2: &[String]) -> Vec<Vec<usize>> {
    let (len1, len2) = (v1.len(), v2.len());
    let mut table = vec![vec![0; len2 + 1]; len1 + 1];
    
    for i in (0..len1).rev() {
        for j in (0..len2).rev() {
            table[i][j] = if v1[i] == v2[j] {
                1 + table[i + 1][j + 1]
            } else {
                max(table[i + 1][j], table[i][j + 1])
            };
        }
    }
    table
}

fn generate_diff(v1: &[String], v2: &[String], table: &[Vec<usize>]) -> String {
    let mut result = String::new();
    let mut i = 0;
    let mut j = 0;
    let mut start_i = 1; // Line numbers are 1-indexed
    let mut start_j = 1;
    let mut chunk1 = Vec::new();
    let mut chunk2 = Vec::new();
    
    while i < v1.len() || j < v2.len() {
        // Lines match
        if i < v1.len() && j < v2.len() && v1[i] == v2[j] {
            // Skip whitespace-only lines that precede differences
            if is_whitespace_before_diff(v1, v2, i, j) {
                chunk1 = add_to_chunk(chunk1, &mut start_i, &v1[i]);
                chunk2 = add_to_chunk(chunk2, &mut start_j, &v2[j]);
                i += 1;
                j += 1;
                continue;
            }
            
            // Flush any pending chunks
            if !chunk1.is_empty() || !chunk2.is_empty() {
                trim_trailing_whitespace(&mut chunk1);
                trim_trailing_whitespace(&mut chunk2);
                result.push_str(&format_chunk(&chunk1, &chunk2, start_i, start_j));
                chunk1.clear();
                chunk2.clear();
            }
            
            i += 1;
            j += 1;
            start_i = i + 1;
            start_j = j + 1;
        }
        // Deletion from v1 or addition to v2
        else if j >= v2.len() || (i < v1.len() && table[i + 1][j] > table[i][j + 1]) {
            chunk1 = add_to_chunk(chunk1, &mut start_i, &v1[i]);
            i += 1;
        } else {
            chunk2 = add_to_chunk(chunk2, &mut start_j, &v2[j]);
            j += 1;
        }}
    
    // Flush remaining chunks
    if !chunk1.is_empty() || !chunk2.is_empty() {
        result.push_str(&format_chunk(&chunk1, &chunk2, start_i, start_j));}
    
    result
}

fn is_whitespace_before_diff(v1: &[String], v2: &[String], i: usize, j: usize) -> bool {
    i + 2 < v1.len()
        && j + 2 < v2.len()
        && v1[i].trim().is_empty()
        && v2[j].trim().is_empty()
        && v1[i + 1] != v2[j + 1]
        && v1[i + 2] != v2[j + 2]
}

fn add_to_chunk(
    mut chunk: Vec<(usize, String)>,
    start_line: &mut usize,
    line: &str,
) -> Vec<(usize, String)> {
    if chunk.is_empty() && line.trim().is_empty() {
        *start_line += 1;
    } else {
        let line_num = *start_line + chunk.len();
        chunk.push((line_num, line.to_string()));
    }
    chunk
}

fn trim_trailing_whitespace(chunk: &mut Vec<(usize, String)>) {
    while let Some((_, line)) = chunk.last() {
        if line.trim().is_empty() {
            chunk.pop();
        } else {
            break;
        }
    }
}

fn format_chunk(
    chunk1: &[(usize, String)],
    chunk2: &[(usize, String)],
    start1: usize,
    start2: usize,
) -> String {
    let op = match (chunk1.is_empty(), chunk2.is_empty()) {
        (true, false) => "a",
        (false, true) => "d",
        _ => "c",
    };
    let range1 = format_range(chunk1, start1);
    let range2 = format_range(chunk2, start2);
    let mut result = format!("{}{}{}\n", range1, op, range2);
    if !chunk1.is_empty() {
        result.push_str(&format_lines(chunk1, "< "));
    }
    if !chunk1.is_empty() && !chunk2.is_empty() {
        result.push_str("---\n");
    }
    if !chunk2.is_empty() {
        result.push_str(&format_lines(chunk2, "> "));
    }
    result
}
fn format_range(chunk: &[(usize, String)], start: usize) -> String {
    match chunk.len() {
        0 => start.to_string(),
        1 => chunk[0].0.to_string(),
        _ => format!("{},{}", chunk[0].0, chunk.last().unwrap().0),
    }
}
fn format_lines(lines: &[(usize, String)], prefix: &str) -> String {
    lines
        .iter()
        .map(|(_, line)| format!("{}{}\n", prefix, line))
        .collect()
}

fn read_file(path: &str) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", path))
        .lines()
        .map(String::from)
        .collect()
}
