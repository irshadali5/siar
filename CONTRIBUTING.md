# Contributing to Siar

Thank you for your interest in contributing to **Siar**! We warmly welcome contributions of all kinds—from security audits and bug fixes to UI enhancements and core P2P protocol optimizations.

---

## ⚠️ AI Development Disclaimer & Security Notice

> **Important**: This project has been heavily developed and iterated on with **AI assistance**. While significant effort has gone into software architecture, P2P networking logic, and cryptographic key management, AI-generated code can occasionally harbor subtle edge cases, security oversights, or unoptimized patterns.
>
> We strongly advise users and developers to **carefully review and audit the code** before deploying or relying on Siar for mission-critical security applications.

Because of this, **security reviews and community contributions are exceptionally valuable to this project!**

---

## 🔒 Security Contributions

If you discover a security vulnerability, cryptographic weakness, or logic flaw:
1. **Reporting**: Please open a confidential security advisory on GitHub or reach out directly to the maintainers before disclosing the issue publicly.
2. **Audits**: Independent security audits, static analysis results, and dependency reviews are always welcome and appreciated.

---

## 🛠 How to Contribute

### 1. Getting Started
- Read the [DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md) to understand the crate structure, building procedures, and development workflows.
- Read [ARCHITECTURE.md](ARCHITECTURE.md) for a deep dive into Siar's serverless Iroh integration, storage choices, and signaling protocols.

### 2. Submitting Pull Requests
- **Branching**: Create a feature branch off `main` (e.g., `feature/mesh-routing-optimization` or `fix/android-permission-handling`).
- **Code Quality**: Ensure your code is formatted (`cargo fmt`) and free of lint warnings (`cargo clippy`).
- **Testing**: Run tests using `cargo test -p siar-core` before submitting your PR.
- **Commit Messages**: Keep commit messages clear, descriptive, and focused.

### 3. Reporting Bugs & Requesting Features
- Use the GitHub Issues tracker to submit bug reports or feature requests.
- Provide step-by-step reproduction instructions and terminal/log outputs whenever possible.

Thank you for helping build a safer, fully open-source, serverless messenger!
