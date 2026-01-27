# Filesystem Transaction Module

This module provides atomic file operations with crash recovery support for the rspack storage layer.

## Overview

The `Transaction` struct enables atomic multi-file operations with automatic recovery from process crashes. All operations are tracked and can be committed or recovered atomically.

## Architecture

```
filesystem/
├── mod.rs                    # Module exports (only Transaction)
└── transaction/
    ├── mod.rs                # Transaction implementation
    └── lock/
        ├── mod.rs            # LockHelper (private)
        ├── state.rs          # StateLock (private)
        └── commit.rs         # CommitLock (private)
```

## Public API

```rust
pub struct Transaction {
    // All fields are private
}

impl Transaction {
    pub async fn new(root: Utf8PathBuf, temp_root: Utf8PathBuf, fs: Arc<dyn FileSystem>) -> Self;
    pub async fn begin(&mut self) -> FSResult<()>;
    pub async fn add_file(&mut self, path: impl AsRef<Utf8Path>, content: &[u8]) -> FSResult<()>;
    pub fn remove_file(&mut self, path: impl AsRef<Utf8Path>);
    pub async fn commit(&mut self) -> FSResult<()>;
    pub fn root(&self) -> &Utf8Path;
    pub fn temp_root(&self) -> &Utf8Path;
}
```

## Lock Files

All lock files are stored in the **root directory**:

### `state.lock`

- **Created**: By `begin()`
- **Content**: Process ID (PID)
- **Purpose**: Prevents concurrent transactions by different processes
- **Format**: Plain text containing PID
- **Lifecycle**: Created by `begin()`, kept until next `begin()` or process crash

### `commit.lock`

- **Created**: By `commit()` before executing operations
- **Content**: Lists of files to add and remove
- **Purpose**: Enables crash recovery
- **Format**:
  ```
  [ADD]
  path/to/file1.txt
  path/to/file2.txt
  [REMOVE]
  path/to/file3.txt
  ```
- **Lifecycle**: Created at commit start, deleted after commit success

## Usage

### Basic Usage

```rust
use rspack_storage::filesystem::Transaction;

// Create transaction
let mut tx = Transaction::new(root_dir, temp_dir, fs).await;

// Start new transaction
tx.begin().await?;

// Add files (written to temp directory)
tx.add_file("scope1/file1.pack", &data1).await?;
tx.add_file("scope1/file2.pack", &data2).await?;

// Mark files for removal
tx.remove_file("scope1/old.pack");

// Commit atomically
tx.commit().await?;
```

### Crash Recovery

```rust
// First process (crashes during commit)
let mut tx = Transaction::new(root, temp, fs).await;
tx.begin().await?;
tx.add_file("a.txt", data).await?;
tx.remove_file("b.txt");
tx.commit().await?;  // Writes commit.lock, then crashes

// Second process (automatic recovery)
let mut tx = Transaction::new(root, temp, fs).await;
// Transaction detects uncommitted work and loads:
// - added_files = {a.txt}
// - removed_files = {b.txt}

tx.commit().await?;  // Complete the transaction
```

## Recovery Logic

When `new()` is called, it checks for incomplete transactions:

| state.lock         | commit.lock   | Action                                   |
| ------------------ | ------------- | ---------------------------------------- |
| ❌ Not exists      | -             | Clean up temp directory                  |
| ✅ Process running | -             | **Panic** (transaction in progress)      |
| ✅ Process dead    | ✅ Exists     | Load files into transaction for recovery |
| ✅ Process dead    | ❌ Not exists | Clean up temp directory                  |

## Transaction Lifecycle

### 1. Create Transaction

```rust
let mut tx = Transaction::new(root, temp, fs).await;
```

- Checks for existing `state.lock`
- If process dead + `commit.lock` exists → load files for recovery
- Otherwise → clean up temp directory

### 2. Begin Transaction

```rust
tx.begin().await?;
```

- Creates `state.lock` with current PID
- Clears `added_files` and `removed_files`
- Removes and recreates temp directory

### 3. Add/Remove Files

```rust
tx.add_file("file.txt", data).await?;  // Writes to temp, tracks in added_files
tx.remove_file("old.txt");              // Tracks in removed_files
```

- `add_file()`: Writes to temp directory, removes from `removed_files` if present
- `remove_file()`: Adds to `removed_files` set

### 4. Commit

```rust
tx.commit().await?;
```

1. Validates `state.lock` matches current process (panics if not)
2. Writes `commit.lock` with all operations
3. Moves files from temp to root
4. Deletes old files from root
5. Removes `commit.lock`
6. Clears `added_files` and `removed_files`

**Note**: `state.lock` is kept after commit to allow multiple commits in the same session.

## File Operations Order

During `commit()`, operations are executed in this order:

1. **Move new files** from temp to root (may overwrite existing files)
2. **Delete old files** from root
3. **Clean up** temp directory
4. **Clear** tracking sets

This order ensures that:

- New files are in place before old files are removed
- If a new file has the same path as an old file, it's moved first (overwriting), then the delete is a no-op

## Edge Cases

### Scenario: Remove then Add Same File

```rust
tx.remove_file("a.txt");
tx.add_file("a.txt", new_data).await?;
// Result: added_files = {a.txt}, removed_files = ∅
// Commit will add the new file (no deletion)
```

### Scenario: Add then Remove Same File

```rust
tx.add_file("a.txt", data).await?;
tx.remove_file("a.txt");
// Result: added_files = {a.txt}, removed_files = {a.txt}
// Commit will add then delete (net effect: deletion)
```

### Scenario: Multiple Begins

```rust
tx.begin().await?;
tx.add_file("a.txt", data).await?;
tx.begin().await?;  // Clears added_files and removed_files
// All previous operations are discarded
```

## Internal Design

### Lock Types (Private)

#### `StateLock`

```rust
struct StateLock {
    pid: u32,
}
```

- Serializes to plain text: `"12345"`
- Methods: `is_running()`, `is_current()`

#### `CommitLock`

```rust
struct CommitLock {
    files_to_add: Vec<Utf8PathBuf>,
    files_to_remove: Vec<Utf8PathBuf>,
}
```

#### `LockHelper`

```rust
struct LockHelper {
    root_dir: Utf8PathBuf,
    fs: Arc<dyn FileSystem>,
}
```

- `state_lock()` → `Option<StateLock>`
- `update_state_lock(Option<&StateLock>)` - `Some` = write, `None` = delete
- `commit_lock()` → `Option<CommitLock>`
- `update_commit_lock(Option<&CommitLock>)` - `Some` = write, `None` = delete

### Transaction Fields

```rust
pub struct Transaction {
    root: Utf8PathBuf,
    temp_root: Utf8PathBuf,
    fs: Arc<dyn FileSystem>,
    lock_helper: LockHelper,              // Manages locks in root directory
    added_files: HashSet<Utf8PathBuf>,    // Files to move from temp to root
    removed_files: HashSet<Utf8PathBuf>,  // Files to delete from root
}
```

## Design Principles

1. **Single Lock Directory**: All locks in root (not temp) for easier recovery
2. **No Exports**: Only `Transaction` is public; locks are implementation details
3. **Simple Recovery**: Load `commit.lock` contents into transaction state
4. **Idempotent**: `commit()` can be called multiple times on recovered transactions
5. **Fail-Safe**: Operations are atomic; either all succeed or all are recoverable

## Error Handling

- **Panic**: When another process holds `state.lock` or PID mismatch
- **FSResult**: File system errors during operations
- **Recovery**: Automatic on `new()` if previous process crashed

## Thread Safety

⚠️ **Not thread-safe**: Use one `Transaction` per process. Multiple processes are prevented by `state.lock`.

## Limitations

1. No nested transactions
2. No rollback after `commit()` starts
3. Requires exclusive process-level access (enforced by `state.lock`)
4. Temp directory must be on same filesystem as root (for atomic moves)

## Testing

Tests require a `FileSystem` mock. See `transaction/mod.rs` for test structure.
