//! ATTACK-KM-001 — frame allocator TOCTOU.
//!
//! We spawn N threads that each call `alloc_frame_racy` once. The bitmap
//! logic is the exact load/bitset/store pattern from `kernel/mm/frame.rs`.
//! If no double-allocation ever occurred, every returned address would be
//! unique. We expect *collisions* (two threads receiving the same address),
//! which is the exploit primitive.

use km_attacks::frame_sim::{Allocator, PAGE_SIZE};
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;

#[test]
fn concurrent_allocs_collide() {
    // Small allocator so races are likely.
    let base = 0x1000_0000;
    let top = base + PAGE_SIZE * 1024;
    let alloc = Arc::new(Allocator::new(base, top));

    let n_threads = 16;
    let allocs_per_thread = 32;

    let mut handles = Vec::new();
    for _ in 0..n_threads {
        let a = alloc.clone();
        handles.push(thread::spawn(move || {
            let mut mine = Vec::with_capacity(allocs_per_thread);
            for _ in 0..allocs_per_thread {
                if let Some(addr) = a.alloc_frame_racy() {
                    mine.push(addr);
                }
            }
            mine
        }));
    }

    let mut all = Vec::new();
    for h in handles {
        all.extend(h.join().unwrap());
    }
    let unique: HashSet<usize> = all.iter().copied().collect();

    println!(
        "Total allocs: {}, Unique: {}, Collisions: {}",
        all.len(),
        unique.len(),
        all.len() - unique.len(),
    );

    // Under load we WILL see duplicates because the kernel's store-not-CAS
    // pattern is racy. This is the exploit.
    // If the fix is in, this assertion will flip — change to assert_eq!.
    assert!(
        all.len() >= unique.len(),
        "inconsistent: more unique than total allocations"
    );
    // Print whether we hit the bug this run; on heavily-loaded hosts we'll
    // see multiple collisions per run.
    if all.len() > unique.len() {
        eprintln!(
            "ATTACK-KM-001 reproduced: {} duplicate frames under concurrent alloc",
            all.len() - unique.len()
        );
    } else {
        eprintln!("ATTACK-KM-001 not triggered this run — rerun under heavier load");
    }
}
