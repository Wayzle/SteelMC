//! Thread-count selection helpers.

use gdt_cpus::{ThreadPriority, set_thread_priority};
use std::num::NonZero;
use std::thread;

/// Assumed parallelism when `available_parallelism` is unavailable.
pub const AVAILABLE_PARALLELISM_FALLBACK: usize = 4;

/// Stack size for debug-build threads with deep density-function call chains.
pub const DEBUG_STACK_SIZE: usize = 8 * 1024 * 1024;

/// Returns the host's available parallelism, or [`AVAILABLE_PARALLELISM_FALLBACK`].
#[must_use]
pub fn available_worker_threads() -> usize {
    thread::available_parallelism().map_or(AVAILABLE_PARALLELISM_FALLBACK, NonZero::get)
}

/// Caps an explicit positive worker count to available parallelism, or uses
/// half the available threads with a minimum target of two.
#[must_use]
pub fn worker_threads_for_available(
    configured_threads: Option<usize>,
    available_threads: usize,
) -> usize {
    let available_threads = available_threads.max(1);
    if let Some(configured_threads) = configured_threads.filter(|&threads| threads > 0) {
        return configured_threads.min(available_threads);
    }

    ((available_threads / 2).max(2)).min(available_threads)
}

/// Custom handler for rayon ThreadPoolBuilder spawn_handler(),
/// allows to set priority for thread.
pub fn custom_spawn_handler(
    priority: ThreadPriority,
) -> impl FnMut(rayon::ThreadBuilder) -> std::io::Result<()> {
    move |thread| {
        let mut b = std::thread::Builder::new();
        if let Some(name) = thread.name() {
            b = b.name(name.to_owned());
        }
        if let Some(stack_size) = thread.stack_size() {
            b = b.stack_size(stack_size);
        }
        b.spawn(move || {
            set_recommended_thread_conf(priority);
            thread.run()
        })?;
        Ok(())
    }
}

/// Sets the recommended thread affinity and priority based on cpu information.
pub fn set_recommended_thread_conf(priority: ThreadPriority) {
    match set_thread_priority(priority) {
        Ok(applied) if applied.degraded() => {
            eprintln!("priority silently weakened: {}", applied);
        }
        Ok(applied) => println!("priority set: {}", applied),
        Err(e) => eprintln!("priority change failed: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::worker_threads_for_available;

    #[test]
    fn explicit_worker_count_is_capped_to_available_threads() {
        assert_eq!(worker_threads_for_available(Some(16), 8), 8);
        assert_eq!(worker_threads_for_available(Some(4), 8), 4);
    }

    #[test]
    fn zero_or_missing_worker_count_uses_auto_default() {
        assert_eq!(worker_threads_for_available(Some(0), 8), 4);
        assert_eq!(worker_threads_for_available(None, 8), 4);
        assert_eq!(worker_threads_for_available(None, 1), 1);
    }
}
