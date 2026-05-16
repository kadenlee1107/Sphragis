# Contributing to Sphragis

Thank you for your interest in contributing. Sphragis is a security-first
bare-metal Rust microkernel targeting government and high-assurance use.
We use a lightweight contribution process.

## Developer Certificate of Origin (DCO)

By contributing, you certify the statements in the [Developer Certificate
of Origin v1.1](https://developercertificate.org). Every commit must be
signed off using `git commit -s` (which adds a `Signed-off-by:` trailer).

The DCO is preferred over a CLA because it does not assign copyright;
it certifies you have the right to contribute the code under our
license (Apache-2.0). Apache-2.0 + DCO is the same model used by the
Linux kernel and most modern open-source infrastructure projects.

## License

All contributions are licensed under Apache-2.0. See [LICENSE](LICENSE).

## Process

1. Open an issue describing the change you intend to make. Brief is fine.
2. Fork and create a feature branch (`fix/<scope>-<short-desc>` or `feat/<scope>-<short-desc>`).
3. Make your changes. Run `cargo build --target aarch64-unknown-none --release` and `cargo clippy --target aarch64-unknown-none --release`; both must pass clean.
4. Run `python3 scripts/qemu_boot_smoke.py` and `python3 scripts/qemu_cave_private_selftest.py`; both must PASS.
5. Open a PR. Include your DCO sign-off (`git commit -s`).
6. Address review feedback. Maintainers will merge.

## Security disclosures

For security issues, please email security@sphragis.dev (or open a
GitHub security advisory if the repo has them enabled). Do not file
public issues for unpatched vulnerabilities.

## Code of conduct

Be respectful. Sphragis is a small project and we want it to stay welcoming.
