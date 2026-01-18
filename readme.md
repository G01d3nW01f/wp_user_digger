# wp_user_digger

`wp_user_digger` is a lightweight **command-line tool written in Rust** for **enumerating publicly visible WordPress user accounts** via the `/?author=` redirect mechanism.

This tool is intended **strictly for internal security audits** and **asset management** of WordPress sites that you own or administrate.

---

## 🔍 Purpose

In environments where multiple WordPress sites are managed under a single system, it is common to:

- Require site owners to submit a list of registered login users
- Periodically check for:
  - Unreported user accounts
  - Legacy or forgotten accounts
  - Unexpected or suspicious users

`wp_user_digger` helps by collecting **externally visible author usernames** and comparing them with declared user lists.

---

## ⚠️ Legal & Ethical Notice

- This tool only uses **publicly accessible WordPress behavior**
- It does **not** attempt authentication or brute force
- Use **only on systems you own or have explicit permission to audit**
- Unauthorized use against third-party systems may violate laws or terms of service

---

## ✨ Features

- Enumerates users via `/?author=N` redirect behavior
- Extracts usernames from `/author/<username>/`
- No HTML parsing (redirect header analysis only)
- Built-in request delay to avoid WAF / Fail2Ban triggers
- Automatic duplicate removal
- Minimal, fast, and safe Rust implementation
- Clear USAGE / HELP output via `clap`

---

## 📦 Installation

### Prerequisites

- Rust 1.70+ recommended
- Cargo

### Build

```bash
git clone https://github.com/your-org/wp_user_digger.git
cd wp_user_digger
cargo build --release
```
### usage
```bash
target/release/wp_user_digger
```
### example
```bash
wp_user_digger \
  --url https://example.com/wordpress \
  --start 1 \
  --end 100 \
  --sleep-sec 2
```
