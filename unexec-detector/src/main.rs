use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process::exit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::sleep;
use std::time::Duration;

// Statistics tracking
static TOTAL_CHILDREN_ANALYZED: AtomicUsize = AtomicUsize::new(0);
static HEURISTIC_DETECTIONS: AtomicUsize = AtomicUsize::new(0);
static FLAG_DETECTIONS: AtomicUsize = AtomicUsize::new(0);
static BOTH_DETECTIONS: AtomicUsize = AtomicUsize::new(0);
static ONLY_HEURISTIC_DETECTIONS: AtomicUsize = AtomicUsize::new(0);
static ONLY_FLAG_DETECTIONS: AtomicUsize = AtomicUsize::new(0);

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <pid> [frequency_hz]", args[0]);
        std::process::exit(1);
    }

    let parent_pid = args[1].parse::<i32>()?;
    // Parse polling frequency, default to 1Hz
    let frequency_hz = if args.len() > 2 {
        args[2].parse::<f64>().unwrap_or(1.0)
    } else {
        1.0
    };

    // Calculate sleep duration in milliseconds
    let sleep_duration = Duration::from_millis((1000.0 / frequency_hz) as u64);

    println!(
        "Monitoring parent PID: {} at {:.2}Hz (polling every {}ms)",
        parent_pid,
        frequency_hz,
        sleep_duration.as_millis()
    );

    // Set up ctrl-c handler for graceful shutdown with statistics
    setup_exit_handler();

    loop {
        // Check if parent process still exists
        if !process_exists(parent_pid) {
            println!("\nParent process {} no longer exists, exiting", parent_pid);
            print_statistics();
            break;
        }

        poll_children(parent_pid)?;

        // Sleep until next poll
        sleep(sleep_duration);

        // Print a separator with timestamp for readability between polls
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        println!(
            "\n----- New poll cycle at {} seconds since epoch -----\n",
            now.as_nanos()
        );
    }

    Ok(())
}

fn process_exists(pid: i32) -> bool {
    Path::new(&format!("/proc/{}", pid)).exists()
}

fn setup_exit_handler() {
    ctrlc::set_handler(move || {
        println!("\nReceived interrupt signal, shutting down...");
        print_statistics();
        exit(0);
    })
    .expect("Error setting Ctrl-C handler");
}

fn print_statistics() {
    println!("\n--- Detection Statistics Summary ---");
    println!(
        "Total children analyzed: {}",
        TOTAL_CHILDREN_ANALYZED.load(Ordering::Relaxed)
    );
    println!(
        "Heuristic detections: {}",
        HEURISTIC_DETECTIONS.load(Ordering::Relaxed)
    );
    println!(
        "Flag-based detections: {}",
        FLAG_DETECTIONS.load(Ordering::Relaxed)
    );
    println!(
        "Detections by both methods: {}",
        BOTH_DETECTIONS.load(Ordering::Relaxed)
    );
    println!(
        "Detections by heuristic only: {}",
        ONLY_HEURISTIC_DETECTIONS.load(Ordering::Relaxed)
    );
    println!(
        "Detections by flag only: {}",
        ONLY_FLAG_DETECTIONS.load(Ordering::Relaxed)
    );
}

fn poll_children(parent_pid: i32) -> Result<(), Box<dyn Error>> {
    // Get the parent's memory map signature
    let parent_maps = get_process_maps(parent_pid)?;
    let parent_exe = get_process_exe(parent_pid)?;
    println!("Parent executable: {}", parent_exe);

    // Find all children of the parent
    let children = get_process_children(parent_pid)?;
    println!("Found {} children", children.len());

    // Examine each child
    for child_pid in children {
        println!("\nAnalyzing child PID: {}", child_pid);
        TOTAL_CHILDREN_ANALYZED.fetch_add(1, Ordering::Relaxed);

        // Get child memory maps and executable
        let child_maps = match get_process_maps(child_pid) {
            Ok(maps) => maps,
            Err(e) => {
                println!("  Error reading maps: {} (process likely terminated)", e);
                continue;
            }
        };

        let child_exe = match get_process_exe(child_pid) {
            Ok(exe) => exe,
            Err(e) => {
                println!("  Error reading exe: {} (process likely terminated)", e);
                continue;
            }
        };

        // Original heuristic approach
        let heuristic_result =
            is_likely_forked_not_execed(&parent_maps, &child_maps, &parent_exe, &child_exe);

        // New direct flag check approach
        let (flag_result, flag_bits) = match is_forked_not_execed(child_pid) {
            Ok((result, bits)) => (result, bits),
            Err(e) => {
                println!("  Error checking PF_FORKNOEXEC flag: {}", e);
                (false, 0)
            }
        };

        // Update statistics
        if heuristic_result {
            HEURISTIC_DETECTIONS.fetch_add(1, Ordering::Relaxed);
        }

        if flag_result {
            FLAG_DETECTIONS.fetch_add(1, Ordering::Relaxed);
        }

        if heuristic_result && flag_result {
            BOTH_DETECTIONS.fetch_add(1, Ordering::Relaxed);
        } else if heuristic_result {
            ONLY_HEURISTIC_DETECTIONS.fetch_add(1, Ordering::Relaxed);
        } else if flag_result {
            ONLY_FLAG_DETECTIONS.fetch_add(1, Ordering::Relaxed);
        }

        if heuristic_result || flag_result {
            println!("  DETECTION RESULTS:");
            println!(
                "    Heuristic detection: {}",
                if heuristic_result {
                    "DETECTED"
                } else {
                    "Not detected"
                }
            );
            println!(
                "    Flag-based detection: {} (raw flags: 0x{:08x})",
                if flag_result {
                    "DETECTED"
                } else {
                    "Not detected"
                },
                flag_bits
            );
            println!("  Child executable: {}", child_exe);
            println!("  Child state: {}", get_process_state(child_pid)?);
            println!(
                "  Memory map similarity: {}%",
                calculate_map_similarity(&parent_maps, &child_maps)
            );
        } else {
            println!("  Normal child process (already exec'd)");
            println!("  Raw process flags: 0x{:08x}", flag_bits);
            println!("  Child executable: {}", child_exe);
            println!(
                "  Memory map similarity: {}%",
                calculate_map_similarity(&parent_maps, &child_maps)
            );
        }
    }

    Ok(())
}

/// Get all direct children of a process
fn get_process_children(pid: i32) -> Result<Vec<i32>, Box<dyn Error>> {
    let mut children = Vec::new();

    // First try the modern way
    let children_path = format!("/proc/{}/task/{}/children", pid, pid);
    if Path::new(&children_path).exists() {
        let content = fs::read_to_string(&children_path)?;
        for child_str in content.split_whitespace() {
            children.push(child_str.parse::<i32>()?);
        }
        return Ok(children);
    }

    // Fallback: scan all processes and check PPIDs
    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        let path = entry.path();

        if let Some(file_name) = path.file_name() {
            if let Ok(other_pid) = file_name.to_string_lossy().parse::<i32>() {
                if other_pid != pid {
                    if let Ok(status) = fs::read_to_string(path.join("status")) {
                        // Extract PPid from status file
                        for line in status.lines() {
                            if line.starts_with("PPid:") {
                                let ppid = line
                                    .split_whitespace()
                                    .nth(1)
                                    .unwrap_or("0")
                                    .parse::<i32>()?;
                                if ppid == pid {
                                    children.push(other_pid);
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(children)
}

/// Read process memory maps
fn get_process_maps(pid: i32) -> Result<Vec<String>, Box<dyn Error>> {
    let maps_path = format!("/proc/{}/maps", pid);
    let file = File::open(maps_path)?;
    let reader = BufReader::new(file);

    let mut maps = Vec::new();
    for line in reader.lines() {
        maps.push(line?);
    }

    Ok(maps)
}

/// Get process executable path
fn get_process_exe(pid: i32) -> Result<String, Box<dyn Error>> {
    let exe_path = format!("/proc/{}/exe", pid);
    let path = fs::read_link(exe_path)?;
    Ok(path.to_string_lossy().to_string())
}

/// Get current process state
fn get_process_state(pid: i32) -> Result<String, Box<dyn Error>> {
    let status_path = format!("/proc/{}/status", pid);
    let content = fs::read_to_string(status_path)?;

    for line in content.lines() {
        if line.starts_with("State:") {
            return Ok(line.trim_start_matches("State:").trim().to_string());
        }
    }

    Ok("Unknown".to_string())
}

/// Calculate memory map similarity percentage
fn calculate_map_similarity(parent_maps: &[String], child_maps: &[String]) -> f64 {
    let parent_set: HashSet<_> = parent_maps.iter().collect();
    let child_set: HashSet<_> = child_maps.iter().collect();

    // Find intersection size
    let intersection = parent_set.intersection(&child_set).count();

    // Calculate Jaccard similarity
    let union_size = parent_set.len() + child_set.len() - intersection;
    if union_size == 0 {
        return 0.0;
    }

    ((intersection as f64 / union_size as f64) * 100.0).round() / 100.0 * 100.0
}

/// Determine if a child is likely forked but not yet exec'd
fn is_likely_forked_not_execed(
    parent_maps: &[String],
    child_maps: &[String],
    parent_exe: &str,
    child_exe: &str,
) -> bool {
    // Criteria 1: Same executable path (hasn't exec'd yet)
    let same_exe = parent_exe == child_exe;

    // Criteria 2: High memory map similarity (CoW pages still shared)
    let similarity = calculate_map_similarity(parent_maps, child_maps);
    let high_similarity = similarity > 90.0;

    // Criteria 3: Roughly the same number of memory mappings
    let similar_map_count = (parent_maps.len() as isize - child_maps.len() as isize).abs() < 5;

    // A freshly forked process that hasn't exec'd yet will have nearly identical memory mappings
    // and the same executable path
    same_exe && high_similarity && similar_map_count
}

/// Determine if a child is a forked but not yet exec'd process
fn is_forked_not_execed(pid: i32) -> Result<(bool, u32), Box<dyn Error>> {
    // Direct check for PF_FORKNOEXEC flag
    let (has_flag, flag_bits) = has_forknoexec_flag(pid)?;
    if has_flag {
        return Ok((true, flag_bits));
    }

    Ok((false, flag_bits))
}

/// Check if process has the PF_FORKNOEXEC flag set
fn has_forknoexec_flag(pid: i32) -> Result<(bool, u32), Box<dyn Error>> {
    // Read /proc/[pid]/stat file which contains process flags
    let stat_path = format!("/proc/{}/stat", pid);
    let stat_content = fs::read_to_string(stat_path)?;

    // Pass the content to our testable function
    parse_forknoexec_flag(&stat_content)
}

/// Parses a stat file content to check for PF_FORKNOEXEC flag
/// This function is separate to facilitate unit testing
fn parse_forknoexec_flag(stat_content: &str) -> Result<(bool, u32), Box<dyn Error>> {
    // Parse the stat file - the 9th field contains the process flags
    let fields: Vec<&str> = stat_content.split_whitespace().collect();
    if fields.len() >= 9 {
        // https://elixir.bootlin.com/linux/v6.13.7/source/include/linux/sched.h#L1679
        // PF_FORKNOEXEC flag is bit 0x00000040
        if let Ok(flags) = fields[8].parse::<u32>() {
            return Ok(((flags & 0x00000040) != 0, flags));
        }
    }

    Err("Could not parse process flags".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_forknoexec_flag_set() {
        // Create a mock stat content with PF_FORKNOEXEC flag set (0x00000040)
        // Format is like: "pid (comm) state ppid ... flags ..."
        let mock_stat = "1234 (test_proc) S 1000 1000 1000 0 0 4194368 1000 0 0 0";
        //                                                       ^ flags with bit 0x2 set

        let result = parse_forknoexec_flag(mock_stat).unwrap();

        // Check that the function correctly identifies the flag is set
        assert_eq!(result.0, true);
        // Check raw flags value
        assert_eq!(result.1, 0x400040);
    }

    #[test]
    fn test_parse_forknoexec_flag_not_set() {
        // Create a mock stat content without PF_FORKNOEXEC flag set
        let mock_stat = "1234 (test_proc) S 1000 1000 1000 0 0 4194304 1000 0 0 0";
        //                                                       ^ flags without bit 0x2 set

        let result = parse_forknoexec_flag(mock_stat).unwrap();

        // Check that the function correctly identifies the flag is not set
        assert_eq!(result.0, false);
        // Check raw flags value
        assert_eq!(result.1, 0x400000);
    }
}
