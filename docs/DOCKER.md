# 🐳 Docker Guide

This document explains how to run app using Docker and Docker Compose.

It is written to be fully usable **without Make**.

If you prefer shortcuts, see the 📄 [docs/MAKE.md](MAKE.md).

---

## Prerequisites

Install:

- **Docker**
- **Docker Compose v2**

Check installation:

```bash
docker --version
docker compose version
```

---

## Quick Start (Docker Compose)

1. Clone the repository:

```bash
git clone <repository-url>
cd <directory-with-repository>
```

2. Create environment file:

```bash
cp .env.example .env
```

Edit `.env`.

3. Build the image:

```bash
docker compose build
```

4. Start the container:

```bash
docker compose up -d
```

5. Watch logs:

```bash
docker compose logs -f
```

### Build

```bash
docker compose build
```
