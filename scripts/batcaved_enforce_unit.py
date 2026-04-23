#!/usr/bin/env python3
"""Followup 3b-enforce: unit test for per-cave proxy enforcement logic.

Directly imports batcaved (no subprocess, no Docker) and exercises:
  - cave_net_register / cave_for_ip / cave_net_unregister_cave
  - cpol_push / cpol_target_allowed
  - The two combined: cave A policy does NOT let cave B's IP through.

Purely logic-level, so it's fast and reproducible without a Docker
daemon. The full end-to-end Docker path is covered by a separate
integration script when a container image is handy.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

import batcaved as bc

def reset():
    with bc.CAVE_POLICY_LOCK: bc.CAVE_POLICY_MIRROR.clear()
    with bc.CAVE_NET_LOCK:    bc.CAVE_NET_IP.clear()

def main():
    reset()

    # 1. Register two caves on different IPs.
    bc.cave_net_register("192.168.215.2", "kali")
    bc.cave_net_register("192.168.215.3", "alpine")
    assert bc.cave_for_ip("192.168.215.2") == "kali"
    assert bc.cave_for_ip("192.168.215.3") == "alpine"
    assert bc.cave_for_ip("10.0.0.99") is None, "unknown ip must be None"
    print("[unit] cave_net_register + cave_for_ip  ok")

    # 2. Push disjoint policies.
    bc.cpol_push("kali",   "github.com",        443, 6)
    bc.cpol_push("kali",   "api.anthropic.com", 443, 6)
    bc.cpol_push("alpine", "httpbin.org",       443, 6)
    print("[unit] cpol_push  ok")

    # 3. Per-cave allowlist semantics.
    assert bc.cpol_target_allowed("kali",   "github.com:443")       is True
    assert bc.cpol_target_allowed("kali",   "api.anthropic.com:443") is True
    assert bc.cpol_target_allowed("kali",   "httpbin.org:443")      is False, \
        "kali should NOT reach alpine's allowlist"
    assert bc.cpol_target_allowed("alpine", "httpbin.org:443")      is True
    assert bc.cpol_target_allowed("alpine", "github.com:443")       is False, \
        "alpine should NOT reach kali's allowlist"
    print("[unit] cpol_target_allowed cross-cave isolation  ok")

    # 4. Wrong proto is denied (rule proto=6, we query something else).
    # Since CONNECT is always TCP (6), this doesn't happen in the real
    # proxy flow, but the function should reject non-TCP by port/proto.
    # The helper only accepts tcp (6) or any (0) implicitly; we verify
    # by adding a UDP rule and checking that host:443 over TCP doesn't
    # match the UDP rule.
    bc.cpol_push("kali", "dns.example.com", 53, 17)
    assert bc.cpol_target_allowed("kali", "dns.example.com:53") is False, \
        "UDP-only rule must NOT allow TCP CONNECT"
    print("[unit] proto-mismatch denied  ok")

    # 5. Port wildcard (port=0) matches any port.
    bc.cpol_push("kali", "any-port.example.com", 0, 6)
    assert bc.cpol_target_allowed("kali", "any-port.example.com:80")   is True
    assert bc.cpol_target_allowed("kali", "any-port.example.com:8443") is True
    print("[unit] port-wildcard  ok")

    # 6. Host wildcard (host="") matches any host on the specified port.
    bc.cpol_push("alpine", "", 80, 6)
    assert bc.cpol_target_allowed("alpine", "anything.local:80") is True
    assert bc.cpol_target_allowed("alpine", "anything.local:81") is False
    print("[unit] host-wildcard  ok")

    # 7. cave_net_unregister_cave drops all that cave's IPs.
    bc.cave_net_register("192.168.215.4", "kali")
    bc.cave_net_unregister_cave("kali")
    assert bc.cave_for_ip("192.168.215.2") is None
    assert bc.cave_for_ip("192.168.215.4") is None
    assert bc.cave_for_ip("192.168.215.3") == "alpine", \
        "other caves' IPs must survive"
    print("[unit] cave_net_unregister_cave leaves other caves  ok")

    # 8. cpol_clear drops only that cave's rules.
    bc.cpol_clear("alpine")
    assert bc.cpol_target_allowed("alpine", "httpbin.org:443") is False
    assert bc.cpol_target_allowed("kali",   "github.com:443")  is True
    print("[unit] cpol_clear preserves other caves  ok")

    reset()
    print("\n[unit] ALL 8 CHECKS OK")
    return 0

if __name__ == "__main__":
    sys.exit(main())
