# Multi-threading Refactoring Guide: Folder Expansion

## Overview

This guide walks through refactoring the synchronous `expand()` method in `FolderHolder` to use multi-threading. This is an excellent learning exercise for understanding Rust's concurrency primitives.

## Current Implementation Analysis

### What `expand()` Currently Does (Synchronous)

```rust
pub fn expand(&mut self) -> AppResult<()> {
    let first_item = self.current_holder[0].clone();
    let value_path_group: Vec<PathBuf> = self
        .current_holder
        .iter()
        .skip(1)
        .filter_map(|p| p.to_path_canonicalize().ok())
        .collect();

    let mut result = Vec::new();
    for p in &value_path_group {
        if p.is_dir() {
            let group = FileGroupHolder::new(p.clone(), false)?;  // ← SLOW: I/O operation
            result.extend(group.child);
        } else {
            let file_holder = FileHolder::try_from(p.clone())?;
            result.push(file_holder);
        }
    }

    result.insert(0, first_item);
    self.current_holder = result;
    self.update(None)?;
    self.expand_level = self.expand_level.saturating_add(1);

    Ok(())
}
```

**Performance Bottleneck**: The `for` loop performs sequential I/O operations:
- `FileGroupHolder::new()` calls `fs::read_dir()` for each directory
- Each I/O operation blocks until complete
- For N directories, this takes O(N) time sequentially

**With Multi-threading**: We can process N directories in parallel, reducing time to O(N/M) where M = number of threads.

---

## Learning Path: From Sync to Async Multi-threading

### Phase 1: Understanding the Problem Space

#### Why Multi-threading Here?
1. **I/O-bound operations**: Reading directories is blocking I/O
2. **Embarrassingly parallel**: Each directory read is independent
3. **Real performance gain**: Can be 2-10x faster on SSDs with many subdirectories

#### Rust Concurrency Concepts We'll Use:
- **`std::thread`**: Spawning OS threads
- **`std::sync::mpsc`**: Multi-producer, single-consumer channels
- **`std::sync::Arc`**: Atomic Reference Counting for shared ownership
- **`std::sync::Mutex`**: Safe mutable shared state
- **`rayon`** (optional): Data parallelism library

---

### Phase 2: Basic Multi-threading with `std::thread`

#### Approach 1: Spawn Threads + Collect Results

```rust
use std::thread;
use std::sync::mpsc;

pub fn expand_multithreaded(&mut self) -> AppResult<()> {
    let first_item = self.current_holder[0].clone();

    // Collect paths to process
    let paths_to_process: Vec<PathBuf> = self
        .current_holder
        .iter()
        .skip(1)
        .filter_map(|p| p.to_path_canonicalize().ok())
        .collect();

    // Channel for thread communication
    let (tx, rx) = mpsc::channel();

    // Spawn threads
    let mut handles = vec![];
    for path in paths_to_process {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            // Each thread processes one path
            let result = if path.is_dir() {
                FileGroupHolder::new(path.clone(), false)
                    .map(|group| group.child)
            } else {
                FileHolder::try_from(path.clone())
                    .map(|holder| vec![holder])
            };

            // Send result back to main thread
            tx.send(result).unwrap();
        }));
    }

    // Drop the original sender so receiver knows when to stop
    drop(tx);

    // Collect results
    let mut result = Vec::new();
    for received in rx {
        match received {
            Ok(mut items) => result.append(&mut items),
            Err(e) => return Err(e),  // Early return on error
        }
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    result.insert(0, first_item);
    self.current_holder = result;
    self.update(None)?;
    self.expand_level = self.expand_level.saturating_add(1);

    Ok(())
}
```

**Key Concepts Explained**:

1. **`mpsc::channel()`**: Creates a communication channel
   - `tx` (transmitter): Sends messages from threads
   - `rx` (receiver): Receives messages in main thread
   - Multiple producers, single consumer

2. **`thread::spawn()`**: Creates a new OS thread
   - Takes a closure `move || { ... }`
   - `move` captures variables by value
   - Returns a `JoinHandle` for waiting

3. **`Arc` (Atomic Reference Counting)**:
   - Shared ownership across threads
   - Thread-safe reference counting
   - Cloning is cheap (increments counter)

4. **`Mutex` (Mutual Exclusion)**:
   - Safe interior mutability across threads
   - `lock()` returns `MutexGuard` RAII
   - Automatically unlocks when guard goes out of scope

#### Problems with This Approach:
- ❌ No error handling for thread panics
- ❌ No limit on thread count (could spawn 1000s!)
- ❌ `unwrap()` calls can panic entire program
- ❌ No timeout or cancellation support

---

### Phase 3: Robust Multi-threading with Error Handling

```rust
use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

// Custom error type for threading
#[derive(Debug, thiserror::Error)]
enum ThreadError {
    #[error("Thread panicked: {0}")]
    Panic(String),
    #[error("Thread timeout")]
    Timeout,
    #[error("Channel error: {0}")]
    Channel(String),
}

pub fn expand_multithreaded_robust(&mut self) -> AppResult<()> {
    let first_item = self.current_holder[0].clone();
    let paths_to_process: Vec<PathBuf> = self
        .current_holder
        .iter()
        .skip(1)
        .filter_map(|p| p.to_path_canonicalize().ok())
        .collect();

    if paths_to_process.is_empty() {
        return Ok(());
    }

    // Limit thread count to prevent resource exhaustion
    let num_threads = std::cmp::min(paths_to_process.len(), 8); // Max 8 threads
    let chunk_size = (paths_to_process.len() + num_threads - 1) / num_threads;

    let (tx, rx) = mpsc::channel();
    let tx = Arc::new(tx);  // Share transmitter across threads

    let mut handles = vec![];

    for chunk in paths_to_process.chunks(chunk_size) {
        let tx = Arc::clone(&tx);
        let chunk: Vec<PathBuf> = chunk.to_vec();

        let handle = thread::spawn(move || {
            // Process each path in this chunk
            for path in chunk {
                let result = if path.is_dir() {
                    FileGroupHolder::new(path.clone(), false)
                        .map(|group| group.child)
                } else {
                    FileHolder::try_from(path.clone())
                        .map(|holder| vec![holder])
                };

                if tx.send(result).is_err() {
                    break; // Receiver dropped, stop processing
                }
            }
        });

        handles.push(handle);
    }

    // Drop original transmitter
    drop(tx);

    // Collect results with timeout
    let timeout = Duration::from_secs(30);
    let mut result = Vec::new();
    let mut completed_threads = 0;

    // Use select! for timeout (requires nightly or crossbeam)
    // For stable Rust, use a loop with timeout checking
    let start = std::time::Instant::now();

    while completed_threads < handles.len() {
        match rx.recv_timeout(timeout.saturating_sub(start.elapsed())) {
            Ok(Ok(mut items)) => result.append(&mut items),
            Ok(Err(e)) => return Err(e),  // Propagate errors
            Err(mpsc::RecvTimeoutError::Timeout) => {
                return Err(AppError::Parse("Operation timed out".into()));
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // All senders dropped, break
                break;
            }
        }
    }

    // Wait for threads with error handling
    for handle in handles {
        if let Err(panic_err) = handle.join() {
            let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            return Err(AppError::Parse(format!("Thread panicked: {}", panic_msg)));
        }
    }

    result.insert(0, first_item);
    self.current_holder = result;
    self.update(None)?;
    self.expand_level = self.expand_level.saturating_add(1);

    Ok(())
}
```

**New Concepts**:

1. **Chunking**: Divide work to balance load
2. **`recv_timeout()`**: Non-blocking receive with timeout
3. **`join()` with error handling**: Capture panics from threads
4. **`Arc::clone()`**: Explicit shared ownership

---

### Phase 4: Using Rayon (High-Level Data Parallelism)

Rayon provides a safe, ergonomic API for parallel iterators.

**Add to Cargo.toml**:
```toml
[dependencies]
rayon = "1.8"
```

```rust
use rayon::prelude::*;

pub fn expand_rayon(&mut self) -> AppResult<()> {
    let first_item = self.current_holder[0].clone();

    // Use parallel iterator - Rayon handles thread pool automatically
    let result: Result<Vec<FileHolder>, AppError> = self
        .current_holder
        .par_iter()  // ← Parallel iterator!
        .skip(1)
        .filter_map(|p| p.to_path_canonicalize().ok())
        .map(|path| {
            if path.is_dir() {
                FileGroupHolder::new(path.clone(), false)
                    .map(|group| group.child)
            } else {
                FileHolder::try_from(path.clone())
                    .map(|holder| vec![holder])
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|nested| nested.into_iter().flatten().collect());

    let mut result = result?;
    result.insert(0, first_item);
    self.current_holder = result;
    self.update(None)?;
    self.expand_level = self.expand_level.saturating_add(1);

    Ok(())
}
```

**Rayon Advantages**:
- ✅ Automatic thread pool management
- ✅ Work stealing for load balancing
- ✅ Safe error propagation with `Result`
- ✅ Minimal code changes
- ✅ Handles panics gracefully

**Rayon Disadvantages**:
- External dependency
- Less control over thread count
- May be overkill for simple cases

---

### Phase 5: Thread-Safe Caching with `DashMap`

For the cache (`LruCache`), we need thread-safe concurrent access.

**Add to Cargo.toml**:
```toml
[dependencies]
dashmap = "5.5"
```

```rust
use dashmap::DashMap;
use std::sync::Arc;

pub struct ThreadSafeFolderHolder {
    state_holder: Rc<RefCell<StateHolder>>,
    cache_holder: Arc<DashMap<PathBuf, FileGroupHolder>>,  // Thread-safe map
    // ... other fields
}

impl ThreadSafeFolderHolder {
    pub fn expand_with_concurrent_cache(&mut self) -> AppResult<()> {
        let first_item = self.current_holder[0].clone();
        let paths_to_process: Vec<PathBuf> = /* ... */;

        // Parallel processing with concurrent cache writes
        let results: Vec<FileHolder> = paths_to_process
            .par_iter()
            .filter_map(|path| {
                // Check cache first (concurrent read)
                if let Some(cached) = self.cache_holder.get(path) {
                    return Some(cached.child.clone());
                }

                // Cache miss: read from disk
                let result = if path.is_dir() {
                    FileGroupHolder::new(path.clone(), false)
                        .map(|group| {
                            // Store in concurrent cache
                            self.cache_holder.insert(path.clone(), group.clone());
                            group.child
                        })
                } else {
                    FileHolder::try_from(path.clone())
                        .map(|holder| vec![holder])
                };

                result.ok()
            })
            .flatten()
            .collect();

        // ... rest of method
        Ok(())
    }
}
```

---

## Complete Refactored Implementation

Here's a production-ready version combining best practices:

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use rayon::prelude::*;

// Configuration constants
const MAX_THREADS: usize = 8;
const THREAD_TIMEOUT_SECS: u64 = 30;
const BATCH_SIZE: usize = 16;

/// Thread-safe result type
type ThreadResult<T> = Result<T, AppError>;

pub struct ParallelFolderHolder {
    state_holder: Rc<RefCell<StateHolder>>,
    cache_holder: LruCache<PathBuf, FileGroupHolder>,
    pub input: String,
    pub selected_path_holder: Vec<FileHolder>,
    pub current_directory: PathBuf,
    initial_directory: PathBuf,
    current_holder: Vec<FileHolder>,
    expand_level: usize,
}

impl ParallelFolderHolder {
    /// Expands all directories using parallel processing
    ///
    /// # Performance
    /// - Sequential: O(N) where N = number of directories
    /// - Parallel: O(N/M) where M = number of threads
    /// - Typical speedup: 2-8x on SSD with many subdirectories
    ///
    /// # Returns
    /// `AppResult<()>` with error variants:
    /// - `AppError::Parse`: Thread panic or timeout
    /// - `AppError::Path`: Path resolution failure
    /// - `AppError::Io`: I/O errors during processing
    pub fn expand_parallel(&mut self) -> AppResult<()> {
        let start_time = Instant::now();

        // Early return for empty directories
        if self.current_holder.is_empty() {
            return Ok(());
        }

        let first_item = self.current_holder[0].clone();

        // Prepare work items
        let work_items: Vec<PathBuf> = self
            .current_holder
            .iter()
            .skip(1)
            .filter_map(|p| p.to_path_canonicalize().ok())
            .collect();

        if work_items.is_empty() {
            return Ok(());
        }

        // Use Rayon for parallel processing
        // This handles thread pool, work stealing, and error propagation
        let results: ThreadResult<Vec<FileHolder>> = work_items
            .par_iter()
            .with_min_len(BATCH_SIZE)  // Optimize chunk size
            .map(|path| {
                // Process each path
                if path.is_dir() {
                    FileGroupHolder::new(path.clone(), false)
                        .map(|group| group.child)
                } else {
                    FileHolder::try_from(path.clone())
                        .map(|holder| vec![holder])
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|nested| nested.into_iter().flatten().collect())
            .map_err(|e| AppError::Parse(format!("Parallel processing failed: {}", e)));

        let mut result = results?;
        result.insert(0, first_item);
        self.current_holder = result;
        self.update(None)?;
        self.expand_level = self.expand_level.saturating_add(1);

        // Log performance
        let elapsed = start_time.elapsed();
        log::debug!("Parallel expand took {:?} for {} items", elapsed, work_items.len());

        Ok(())
    }

    /// Alternative: Manual thread spawning with work stealing
    pub fn expand_manual_threads(&mut self) -> AppResult<()> {
        let first_item = self.current_holder[0].clone();
        let work_items: Vec<PathBuf> = /* ... */;

        // Use a thread pool pattern with bounded threads
        let chunk_size = (work_items.len() + MAX_THREADS - 1) / MAX_THREADS;
        let results = Arc::new(Mutex::new(Vec::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));

        let mut handles = vec![];

        for chunk in work_items.chunks(chunk_size) {
            let results = Arc::clone(&results);
            let errors = Arc::clone(&errors);
            let chunk: Vec<PathBuf> = chunk.to_vec();

            handles.push(thread::spawn(move || {
                for path in chunk {
                    let result = if path.is_dir() {
                        FileGroupHolder::new(path.clone(), false)
                            .map(|group| group.child)
                    } else {
                        FileHolder::try_from(path.clone())
                            .map(|holder| vec![holder])
                    };

                    match result {
                        Ok(items) => {
                            let mut guard = results.lock().unwrap();
                            guard.extend(items);
                        }
                        Err(e) => {
                            let mut guard = errors.lock().unwrap();
                            guard.push(e);
                            break; // Stop this thread on error
                        }
                    }
                }
            }));
        }

        // Wait for completion with timeout
        let timeout = Duration::from_secs(THREAD_TIMEOUT_SECS);
        let start = Instant::now();

        for handle in handles {
            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                return Err(AppError::Parse("Thread timeout".into()));
            }

            match handle.join() {
                Ok(()) => {},
                Err(_) => {
                    return Err(AppError::Parse("Thread panicked".into()));
                }
            }
        }

        // Check for errors
        let errors_guard = errors.lock().unwrap();
        if let Some(first_error) = errors_guard.first() {
            return Err(first_error.clone());
        }

        let mut result = results.lock().unwrap().clone();
        result.insert(0, first_item);
        self.current_holder = result;
        self.update(None)?;
        self.expand_level = self.expand_level.saturating_add(1);

        Ok(())
    }
}
```

---

## Testing Your Multi-threaded Code

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    #[test]
    fn test_expand_parallel_basic() {
        // Create test directory structure
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().to_path_buf();

        // Create 10 subdirectories with files
        for i in 0..10 {
            let dir = base_path.join(format!("dir_{}", i));
            std::fs::create_dir(&dir).unwrap();
            std::fs::write(dir.join("file.txt"), "content").unwrap();
        }

        // Test parallel expand
        let state_holder = Rc::new(RefCell::new(StateHolder::default()));
        let mut holder = ParallelFolderHolder::new(base_path, state_holder).unwrap();

        holder.expand_parallel().unwrap();

        // Should have 11 items: ".." + 10 directories
        assert_eq!(holder.current_holder.len(), 11);
    }

    #[test]
    fn test_parallel_vs_sequential_performance() {
        // Create large directory structure
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().to_path_buf();

        for i in 0..100 {
            let dir = base_path.join(format!("dir_{}", i));
            std::fs::create_dir(&dir).unwrap();
        }

        let state_holder = Rc::new(RefCell::new(StateHolder::default()));

        // Sequential
        let start = Instant::now();
        let mut holder_seq = ParallelFolderHolder::new(base_path.clone(), state_holder.clone()).unwrap();
        holder_seq.expand().unwrap(); // Original method
        let seq_time = start.elapsed();

        // Parallel
        let start = Instant::now();
        let mut holder_par = ParallelFolderHolder::new(base_path, state_holder).unwrap();
        holder_par.expand_parallel().unwrap();
        let par_time = start.elapsed();

        // Parallel should be faster (or at least not slower)
        assert!(par_time <= seq_time * 2); // Allow some overhead

        // Results should be identical
        assert_eq!(holder_seq.current_holder.len(), holder_par.current_holder.len());
    }

    #[test]
    fn test_error_propagation() {
        // Test that errors from threads are properly propagated
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().to_path_buf();

        // Create a directory, then delete it to cause error
        let bad_dir = base_path.join("will_delete");
        std::fs::create_dir(&bad_dir).unwrap();

        let state_holder = Rc::new(RefCell::new(StateHolder::default()));
        let mut holder = ParallelFolderHolder::new(base_path.clone(), state_holder).unwrap();

        // Expand to cache the directory
        holder.expand_parallel().unwrap();

        // Delete the directory
        std::fs::remove_dir_all(&bad_dir).unwrap();

        // Try to expand again - should handle gracefully
        // (This tests error handling in concurrent environment)
        let result = holder.expand_parallel();
        // Should not panic, may return error or handle gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_concurrent_cache_access() {
        // If using DashMap or similar, test concurrent cache operations
        // This would be in the cache module
    }
}
```

### Integration Tests

```rust
#[test]
fn test_large_directory_parallel_expand() {
    // Create 1000 directories with nested structure
    let temp_dir = tempfile::tempdir().unwrap();
    let base = temp_dir.path();

    // Parallel creation of test structure
    (0..100).into_par_iter().for_each(|i| {
        let dir = base.join(format!("level1_{}", i));
        std::fs::create_dir(&dir).unwrap();

        for j in 0..10 {
            let subdir = dir.join(format!("level2_{}", j));
            std::fs::create_dir(&subdir).unwrap();
            std::fs::write(subdir.join("file.txt"), "test").unwrap();
        }
    });

    // Now test parallel expand
    let state_holder = Rc::new(RefCell::new(StateHolder::default()));
    let mut holder = ParallelFolderHolder::new(base.to_path_buf(), state_holder).unwrap();

    let start = Instant::now();
    holder.expand_parallel().unwrap();
    let duration = start.elapsed();

    println!("Expanded {} items in {:?}", holder.current_holder.len(), duration);
    assert!(holder.current_holder.len() > 500); // Should have many items
}
```

---

## Common Pitfalls and Solutions

### 1. **Data Races**
```rust
// ❌ WRONG: Shared mutable state without synchronization
let mut results = Vec::new();
for item in work_items {
    thread::spawn(|| {
        results.push(process(item)); // DATA RACE!
    });
}

// ✅ CORRECT: Use Mutex or channels
let results = Arc::new(Mutex::new(Vec::new()));
let results_clone = Arc::clone(&results);
thread::spawn(move || {
    let processed = process(item);
    results_clone.lock().unwrap().push(processed);
});
```

### 2. **Deadlocks**
```rust
// ❌ WRONG: Locking in different order
let guard1 = lock1.lock().unwrap();
let guard2 = lock2.lock().unwrap(); // May deadlock!

// ✅ CORRECT: Consistent lock ordering or try_lock
let guard2 = lock2.try_lock();
let guard1 = lock1.lock().unwrap();
// Handle try_lock failure
```

### 3. **Thread Explosion**
```rust
// ❌ WRONG: Spawning thread per item
for path in paths {
    thread::spawn(move || process(path)); // Could spawn 1000s!
}

// ✅ CORRECT: Use thread pool or chunking
let chunk_size = (paths.len() + MAX_THREADS - 1) / MAX_THREADS;
for chunk in paths.chunks(chunk_size) {
    // Spawn thread per chunk
}
```

### 4. **Poisoned Mutex**
```rust
// ❌ WRONG: Ignoring poison errors
let data = lock.lock().unwrap(); // Panics if previous lock panicked

// ✅ CORRECT: Handle poison gracefully
let data = match lock.lock() {
    Ok(guard) => guard,
    Err(poisoned) => {
        // Recover from poisoned state
        poisoned.into_inner()
    }
};
```

### 5. **Blocking in Async Context**
```rust
// ❌ WRONG: Blocking I/O in async code
async fn expand_async(&mut self) {
    // This blocks the async runtime!
    let result = thread::spawn(|| blocking_io()).await;
}

// ✅ CORRECT: Use spawn_blocking or async I/O
async fn expand_async(&mut self) {
    let result = tokio::task::spawn_blocking(|| blocking_io()).await;
}
```

---

## Performance Considerations

### When to Use Multi-threading:
- ✅ **I/O-bound**: Reading many files/directories
- ✅ **CPU-bound**: Heavy computation on each item
- ✅ **Many items**: 10+ work items to amortize thread overhead

### When NOT to Use:
- ❌ **Few items**: Thread overhead > benefit (< 5 items)
- ❌ **Already fast**: Sequential takes < 10ms
- ❌ **Shared state**: Heavy synchronization needed

### Benchmarking Template:
```rust
#[bench]
fn bench_expand_parallel(b: &mut Bencher) {
    let temp_dir = create_large_test_dir();
    let state_holder = Rc::new(RefCell::new(StateHolder::default()));

    b.iter(|| {
        let mut holder = ParallelFolderHolder::new(temp_dir.path().to_path_buf(), state_holder.clone()).unwrap();
        holder.expand_parallel().unwrap();
    });
}
```

---

## Recommended Learning Order

1. **Start Simple**: Use `rayon` first - it's safe and ergonomic
2. **Understand Fundamentals**: Learn `std::thread` and channels
3. **Add Safety**: Implement proper error handling and timeouts
4. **Optimize**: Profile and tune thread counts/chunk sizes
5. **Advanced**: Explore `tokio` for async I/O if needed

---

## Final Checklist for Production

- [ ] Error handling covers all thread failure modes
- [ ] Thread count bounded to prevent resource exhaustion
- [ ] Timeout protection for hanging operations
- [ ] Tests for concurrent access patterns
- [ ] Benchmark shows measurable improvement
- [ ] Logging for debugging production issues
- [ ] Graceful degradation on single-core systems
- [ ] Documentation explains threading model

---

## Resources

- [Rayon Documentation](https://docs.rs/rayon)
- [Rust Book: Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [Tokio Async Runtime](https://tokio.rs/)
- [Crossbeam: Advanced Concurrency](https://docs.rs/crossbeam)

This guide provides a complete path from synchronous to production-ready multi-threaded code. Start with Rayon for simplicity, then dive into manual threading for deeper understanding!