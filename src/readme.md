# Bitcoin Address Checker

A high-performance Rust tool for checking if Bitcoin addresses from a text file exist in a SQLite database. Optimized for speed and designed for repeated execution in address discovery workflows.

## 🎯 What It Does

This tool performs the following operations:

1. **Opens a SQLite database** in read-only mode (default: `btc_addresses.db`)
2. **Reads Bitcoin addresses** from a text file, one per line (default: `addressonly.txt`)
3. **Checks each address** against the database using optimized batch queries
4. **Reports all matching addresses** to stderr with clear formatting
5. **Exits with appropriate code**:
   - `0` if no addresses found
   - `1` if one or more addresses found
   - `2` if an error occurred

## 🚀 Features

### Speed Optimizations

- **Read-only database access**: Uses `SQLITE_OPEN_READ_ONLY` flag for faster queries
- **No mutex locking**: `SQLITE_OPEN_NO_MUTEX` eliminates thread synchronization overhead
- **Memory-mapped I/O**: Large file buffer (1MB) for efficient text file reading
- **Batch processing**: Processes 1000 addresses per SQL query to minimize database round-trips
- **SQLite performance tuning**: Optimized PRAGMA settings for read-only workloads
- **Prepared statement caching**: Reuses compiled SQL statements
- **Early termination**: Can stop on first match if modified
- **LTO compilation**: Link-time optimization for smaller, faster binary

### SQLite Optimizations Applied
```sql
PRAGMA journal_mode = OFF;        -- No journaling for read-only
PRAGMA synchronous = OFF;         -- No disk sync needed
PRAGMA temp_store = MEMORY;       -- Use RAM for temp tables
PRAGMA mmap_size = 268435456;     -- 256MB memory-mapped I/O
PRAGMA page_size = 4096;          -- Optimal page size
PRAGMA cache_size = -64000;       -- 64MB page cache

## 📋 Requirements

### Database Schema

Your SQLite database must have a table with the following structure:

sql
CREATE TABLE IF NOT EXISTS addresses (
address TEXT PRIMARY KEY NOT NULL
);

-- Recommended: Create an index for faster lookups
CREATE INDEX IF NOT EXISTS idx_address ON addresses(address);

### Input File Format

The text file should contain one Bitcoin address per line:


1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX
1HLoD9E4SDFFPDiYfNYnkBLQ85Y51J3Zb1

- Empty lines are ignored
- Leading/trailing whitespace is trimmed
- UTF-8 encoding expected

## 🛠️ Installation

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Build from Source

bash
# Clone or download the project
cd btc_address_checker

# Build optimized release version
cargo build --release

# Binary will be at: ./target/release/btc_address_checker

### Install Globally (Optional)

bash
cargo install --path .

## 📖 Usage

### Basic Usage

bash
# Use default files (btc_addresses.db and addressonly.txt)
./target/release/btc_address_checker

# Specify custom database and text file
./target/release/btc_address_checker /path/to/database.db /path/to/addresses.txt

### Command-Line Options


btc_address_checker [DATABASE_PATH] [TEXT_FILE_PATH]

| Argument | Default | Description |
|----------|---------|-------------|
| `DATABASE_PATH` | `btc_addresses.db` | Path to SQLite database file |
| `TEXT_FILE_PATH` | `addressonly.txt` | Path to text file with addresses |

### Examples

**Check with default files:**
bash
./btc_address_checker

**Check with custom database:**
bash
./btc_address_checker my_btc_db.db

**Check with custom database and text file:**
bash
./btc_address_checker wallets.db addresses_to_check.txt

**Use in a script:**
bash
#!/bin/bash
if ./btc_address_checker custom.db check.txt; then
echo "No matches found - continue searching"
else
echo "Match found! Check the output above"
fi

**Continuous checking loop:**
bash
#!/bin/bash
while true; do
./btc_address_checker
if [ $? -eq 1 ]; then
echo "FOUND! Stopping..."
break
fi
sleep 1
done

## 📤 Output

### Success - Addresses Found


✓ Found 3 address(es) in database:
  → 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
  → 12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX
  → 1HLoD9E4SDFFPDiYfNYnkBLQ85Y51J3Zb1
**Exit Code:** `1`

### Success - No Addresses Found


✗ No addresses found in database
**Exit Code:** `0`

### Error


Error: unable to open database file: btc_addresses.db
**Exit Code:** `2`

## 🔢 Exit Codes

| Code | Meaning | Use Case |
|------|---------|----------|
| `0` | No addresses found | Safe to continue searching |
| `1` | One or more addresses found | Match detected! |
| `2` | Error occurred | Check error message |

## ⚡ Performance Tips

### Database Optimization

1. **Use an index:**
```sql
   CREATE INDEX idx_address ON addresses(address);
