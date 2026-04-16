//! ATTACK-KM-022 — futex hash-bucket exhaustion.
//!
//! Find 32 distinct 4-byte-aligned addresses that hash into the same bucket
//! under `bucket_index`. The kernel's wait queue caps at 32 waiters per
//! bucket, so a single cave with this many distinct uaddrs can DoS any other
//! user of that bucket.

use km_attacks::futex_sim::{bucket_index, NUM_BUCKETS, WAITERS_PER_BUCKET};

#[test]
fn find_collision_set_per_bucket() {
    let mut per_bucket: Vec<Vec<u64>> = vec![Vec::new(); NUM_BUCKETS];
    // Brute-force: enumerate addresses and stash up to 64 per bucket.
    let mut uaddr: u64 = 0x1_0000_0000;
    let mut examined = 0;
    while per_bucket.iter().all(|v| v.len() < 64) && examined < 5_000_000 {
        let bi = bucket_index(uaddr);
        if per_bucket[bi].len() < 64 {
            per_bucket[bi].push(uaddr);
        }
        uaddr = uaddr.wrapping_add(4);
        examined += 1;
    }

    let mut full_buckets = 0;
    for (i, v) in per_bucket.iter().enumerate() {
        if v.len() >= WAITERS_PER_BUCKET {
            full_buckets += 1;
            if full_buckets == 1 {
                eprintln!(
                    "ATTACK-KM-022: bucket {} has {} collisions; first 5: {:x?}",
                    i,
                    v.len(),
                    &v[..5]
                );
            }
        }
    }

    assert!(
        full_buckets > 0,
        "expected to saturate at least one bucket within 5M probes"
    );
}
