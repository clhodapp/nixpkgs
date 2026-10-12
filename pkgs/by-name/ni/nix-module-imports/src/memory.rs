//! How much memory the thread that asks has in use, and a limit on it.
//!
//! A module file is read by one thread, and everything that the reading
//! makes is made and dropped on that thread. So the memory that a thread
//! has in use while it reads a file is the memory that the file takes.
//! The allocator counts it and notes when it passes the limit of the
//! thread. The reading asks `over` often and stops the file then, which
//! bounds the memory of the whole run by the limit times the number of
//! files that are read at a time, however long a file takes.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    /// The bytes that this thread has asked for and not given back. A
    /// thread that drops what another thread made goes below zero.
    static IN_USE: Cell<isize> = const { Cell::new(0) };
    /// The count at which this thread is over its limit.
    static LIMIT: Cell<isize> = const { Cell::new(isize::MAX) };
    /// Whether the count has passed the limit since the limit was set.
    static OVER: Cell<bool> = const { Cell::new(false) };
}

/// The allocator of the system, with a count of the bytes in use for
/// each thread.
pub struct Counting;

fn add(bytes: isize) {
    // A thread that is being torn down has no count any more.
    let _ = IN_USE.try_with(|in_use| {
        let now = in_use.get() + bytes;
        in_use.set(now);
        if bytes > 0 {
            let _ = LIMIT.try_with(|limit| {
                if now > limit.get() {
                    let _ = OVER.try_with(|over| over.set(true));
                }
            });
        }
    });
}

// SAFETY: every call goes to the allocator of the system unchanged.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        add(layout.size() as isize);
        // SAFETY: the caller upholds the contract of `alloc`.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        add(-(layout.size() as isize));
        // SAFETY: the caller upholds the contract of `dealloc`.
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        add(layout.size() as isize);
        // SAFETY: the caller upholds the contract of `alloc_zeroed`.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        add(new_size as isize - layout.size() as isize);
        // SAFETY: the caller upholds the contract of `realloc`.
        unsafe { System.realloc(pointer, layout, new_size) }
    }
}

/// From now on this thread may take `bytes` more than it has in use
/// now. Zero is no limit.
pub fn limit(bytes: usize) {
    let now = IN_USE.try_with(Cell::get).unwrap_or(0);
    let _ = LIMIT.try_with(|limit| {
        limit.set(if bytes == 0 {
            isize::MAX
        } else {
            now.saturating_add(bytes as isize)
        });
    });
    let _ = OVER.try_with(|over| over.set(false));
}

/// The bytes that this thread has in use.
#[cfg(test)]
pub fn in_use() -> isize {
    IN_USE.try_with(Cell::get).unwrap_or(0)
}

/// Whether this thread has passed its limit since `limit` was called.
pub fn over() -> bool {
    OVER.try_with(Cell::get).unwrap_or(false)
}
