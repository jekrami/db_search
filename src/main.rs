use rusqlite::{Connection, OpenFlags};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

const DEFAULT_DB_PATH: &str = "btc_addresses.db";
const DEFAULT_TXT_PATH: &str = "addressonly.txt";
const BATCH_SIZE: usize = 1000;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let db_path = args.get(1).map(|s| s.as_str()).unwrap_or(DEFAULT_DB_PATH);
    let txt_path = args.get(2).map(|s| s.as_str()).unwrap_or(DEFAULT_TXT_PATH);
    
    match check_addresses(db_path, txt_path) {
        Ok(found_addresses) => {
            if !found_addresses.is_empty() {
                eprintln!("✓ Found {} address(es) in database:", found_addresses.len());
                for addr in &found_addresses {
                    eprintln!("  → {}", addr);
                }
                process::exit(1);
            } else {
                eprintln!("✗ No addresses found in database");
                process::exit(0);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(2);
        }
    }
}

fn check_addresses(db_path: &str, txt_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Open database in read-only mode with optimizations
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    // Set busy timeout to handle disk I/O contention (5 seconds)
    conn.busy_timeout(std::time::Duration::from_secs(5))?;

    // Set SQLite optimizations for read-only access with WAL support
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA temp_store = MEMORY;
         PRAGMA mmap_size = 268435456;
         PRAGMA cache_size = -64000;
         PRAGMA query_only = ON;
         PRAGMA locking_mode = NORMAL;
         PRAGMA read_uncommitted = 1;"
    )?;
    
    // Read addresses from file
    let file = File::open(txt_path)?;
    let reader = BufReader::with_capacity(1024 * 1024, file); // 1MB buffer
    
    let mut addresses = Vec::new();
    for line in reader.lines() {
        let address = line?.trim().to_string();
        if !address.is_empty() {
            addresses.push(address);
        }
    }
    
    if addresses.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut found_addresses = Vec::new();
    
    // Process in batches for better performance
    for chunk in addresses.chunks(BATCH_SIZE) {
        let mut batch_found = check_batch(&conn, chunk)?;
        found_addresses.append(&mut batch_found);
    }
    
    Ok(found_addresses)
}

fn check_batch(conn: &Connection, addresses: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Build parameterized query for batch checking
    let placeholders = vec!["?"; addresses.len()].join(",");
    let query = format!(
        "SELECT address FROM addresses WHERE address IN ({})",
        placeholders
    );
    
    let mut stmt = conn.prepare_cached(&query)?;
    
    // Convert addresses to rusqlite parameters
    let params: Vec<&dyn rusqlite::ToSql> = addresses
        .iter()
        .map(|s| s as &dyn rusqlite::ToSql)
        .collect();
    
    let mut found = Vec::new();
    let mut rows = stmt.query(&params[..])?;
    
    while let Some(row) = rows.next()? {
        let address: String = row.get(0)?;
        found.push(address);
    }
    
    Ok(found)
}
