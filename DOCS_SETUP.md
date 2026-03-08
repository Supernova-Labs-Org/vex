# vex Documentation Setup

This project uses mkdocs with Material theme to serve documentation.

## Quick Start

### 1. Install Dependencies

```bash
make docs-install
```

Or manually:

```bash
pip install -r docs-requirements.txt
```

### 2. Serve Locally

```bash
make docs-serve
```

Documentation will be available at: **http://localhost:8000**

### 3. Build Static Site

```bash
make docs-build
```

Output goes to `site/` directory (ready for deployment).

## Documentation Structure

```
docs/
├── INDEX.md              # Homepage
├── GETTING_STARTED.md    # Quick start guide
├── CLI_REFERENCE.md      # CLI options reference
├── EXAMPLES.md           # Usage examples
├── METRICS.md            # Metrics explanation
└── README.md             # Index page
```

## Configuration

- **mkdocs.yml** - Main configuration file
- **docs-requirements.txt** - Python dependencies
- **Makefile** - Build targets for docs and project

## Features

- Material theme with dark mode support
- Search functionality
- Code highlighting
- Responsive design
- Offline support

## Build Commands

```bash
# Serve documentation (watch mode)
make docs-serve

# Build static site
make docs-build

# Clean build artifacts
make docs-clean

# Install dependencies
make docs-install
```

## Customization

Edit `mkdocs.yml` to:
- Change site name/description
- Modify navigation structure
- Configure theme colors
- Add plugins

Edit `docs/*.md` files to:
- Update content
- Fix links
- Add new sections

## Deployment

### GitHub Pages

Add to GitHub Actions workflow:

```yaml
- name: Build docs
  run: pip install -r docs-requirements.txt && mkdocs build

- name: Deploy to GitHub Pages
  uses: peaceiris/actions-gh-pages@v3
  with:
    github_token: ${{ secrets.GITHUB_TOKEN }}
    publish_dir: ./site
```

### Other Platforms

1. Run: `make docs-build`
2. Deploy `site/` directory to your hosting

## All Available Commands

```bash
# Project build commands
make build              # Build debug binary
make release            # Build optimized release binary
make test               # Run tests
make clean              # Clean all build artifacts

# Documentation commands
make docs-install       # Install mkdocs dependencies
make docs-serve         # Serve docs locally
make docs-build         # Build static site
make docs-clean         # Clean docs artifacts

# Help
make help               # Show all available commands
```

## Troubleshooting

**Python version issues:**
```bash
python3 -m pip install -r docs-requirements.txt
```

**Port 8000 in use:**
```bash
mkdocs serve --dev-addr 0.0.0.0:8001
```

**Update dependencies:**
```bash
pip install --upgrade -r docs-requirements.txt
```

**Makefile not found:**
Make sure you're in the project root directory.
