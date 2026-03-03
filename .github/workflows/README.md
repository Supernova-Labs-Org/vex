# GitHub Workflows

Automated CI/CD pipelines for vex project.

## Workflows

### docs.yml - Documentation Build & Deploy

**Triggers:** Push to main/master or PR, when docs files change

**Steps:**
1. Checkout code
2. Setup Python 3.12
3. Install mkdocs dependencies
4. Build documentation with mkdocs
5. Deploy to GitHub Pages (on push to main/master)

**Artifacts:** Uploads built site to GitHub Pages

**Requirements:**
- Enable GitHub Pages in repository settings
- Set "Build and deployment" source to "GitHub Actions"

### ci.yml - Code Quality & Testing

**Triggers:** Push to main/master or any PR

**Jobs:**

#### Test
- Run cargo tests
- Run clippy linting
- Check code formatting

#### Documentation Check
- Build docs with strict mode
- Check for broken links (optional)

#### Build Release
- Build release binary

**Caching:**
- Cargo registry, index, and build artifacts cached
- Speeds up subsequent runs

## Setup Instructions

### 1. GitHub Pages Configuration

Go to repository Settings → Pages:
- Source: Deploy from a branch → GitHub Actions

### 2. Trigger First Deployment

Push to main/master branch:
```bash
git push origin main
```

Documentation will be available at: `https://<username>.github.io/vex/`

### 3. Monitor Builds

View workflow status in:
- Actions tab in GitHub
- Pull request status checks

## Configuration

### Update Documentation URL

In `mkdocs.yml`, update:
```yaml
site_url: https://<username>.github.io/vex/
```

### Enable/Disable Workflows

Toggle in repository Actions settings or edit workflow files.

### Modify Branches

Edit branch triggers in workflow files if using different branch names.

## Customization

### Add More CI Checks

Edit `ci.yml` to add:
- Security scanning
- Code coverage
- Additional linters

### Deploy to Custom Domain

In `mkdocs.yml`:
```yaml
site_url: https://docs.example.com/
```

Then configure custom domain in GitHub Pages settings.

### Scheduled Builds

Add schedule trigger to `docs.yml`:
```yaml
schedule:
  - cron: '0 0 * * 0'  # Weekly
```

## Troubleshooting

### Deployment Failed

1. Check Actions tab for error details
2. Verify GitHub Pages source is set to "GitHub Actions"
3. Ensure `docs-requirements.txt` is present
4. Check mkdocs.yml syntax

### Tests Failing

1. View test output in Actions
2. Run locally: `cargo test`
3. Fix issues and push

### Documentation Not Updating

1. Verify workflow runs completed successfully
2. Clear browser cache
3. Check GitHub Pages custom domain configuration

## Useful Commands

View workflow status locally:
```bash
# Show recent runs
gh run list

# View specific run details
gh run view <run-id>

# View workflow logs
gh run view <run-id> --log
```
