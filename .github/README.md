# GitHub Actions Workflows

This directory contains automated build workflows for Rustris.

## Workflows

### `build.yml` - Continuous Integration
**Triggers:** Push to main/master/develop, Pull Requests

Builds the application on every push to verify builds work. Useful for:
- Testing changes before merging
- Ensuring builds don't break
- Getting build artifacts for testing (stored for 7 days)

**Note:** ARM64 builds are commented out by default since GitHub free tier doesn't include ARM64 runners. Uncomment if you have access to ARM64 runners.

### `release.yml` - Release Builds
**Triggers:** Git tags starting with `v` (e.g., `v0.1.0`), Manual dispatch

Creates release builds for both x86_64 and ARM64 architectures and publishes them to GitHub Releases.

## How to Create a Release

### Method 1: Using Git Tags (Recommended)

1. **Update version in `Cargo.toml`**:
   ```toml
   version = "0.2.0"
   ```

2. **Commit and push changes**:
   ```bash
   git add Cargo.toml
   git commit -m "Bump version to 0.2.0"
   git push
   ```

3. **Create and push tag**:
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```

4. **Wait for GitHub Actions to build** (15-30 minutes)
   - Go to: `https://github.com/your-username/rustris/actions`
   - Watch the "Build Release" workflow
   - Once complete, check Releases tab

### Method 2: Manual Trigger

1. Go to your repository on GitHub
2. Click "Actions" tab
3. Select "Build Release" workflow
4. Click "Run workflow" button
5. Select branch and click "Run workflow"

## ARM64 Builds

### GitHub-Hosted ARM64 Runners

GitHub offers ARM64 runners on:
- **Team and Enterprise plans**: Native `ubuntu-22.04-arm64` runners
- **Free tier**: Not included

### Options for Free Tier Users

1. **Self-hosted ARM64 runner**: Set up your own ARM64 runner
   - [GitHub Self-Hosted Runners Guide](https://docs.github.com/en/actions/hosting-your-own-runners)
   - Requires ARM64 hardware (Raspberry Pi 4+, ARM server, etc.)

2. **Comment out ARM64 builds**: Remove ARM64 from the matrix in workflows

3. **Use Docker locally**: Build ARM64 with Docker on your machine (experimental)

4. **Third-party services**:
   - [Cirrus CI](https://cirrus-ci.org/) - Free ARM64 builds for open source
   - [Buildjet](https://buildjet.com/) - Faster GitHub Actions runners

## Customizing Workflows

### Add more architectures
Edit the `matrix.platform` section:
```yaml
matrix:
  platform:
    - os: ubuntu-22.04
      target: x86_64-unknown-linux-gnu
      arch: amd64
    - os: ubuntu-22.04-arm64  # Requires Team/Enterprise
      target: aarch64-unknown-linux-gnu
      arch: arm64
```

### Change trigger conditions
Edit the `on:` section:
```yaml
on:
  push:
    tags:
      - 'v*'  # Only version tags
  workflow_dispatch:  # Allow manual trigger
```

### Change bundled formats
Edit the build command:
```bash
cargo tauri build --bundles deb,appimage,rpm
```

Available bundles: `deb`, `appimage`, `rpm`, `dmg` (macOS), `nsis` (Windows)

## Troubleshooting

### Build fails on dependencies
- Check `Install dependencies` step has all required packages
- System dependencies must match Tauri requirements

### ARM64 runner not available
- Upgrade to GitHub Team/Enterprise, or
- Use self-hosted runner, or
- Comment out ARM64 from matrix

### Release not created
- Ensure you have write permissions to create releases
- Check that `GITHUB_TOKEN` has `contents: write` permission
- Verify tag matches pattern `v*` (e.g., v0.1.0)

### Artifacts not uploaded
- Check `if-no-files-found` setting
- Verify file paths in `path:` match actual build output
- Check build logs for actual file locations

## Testing Locally

Before pushing tags, test the build locally:

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt-get install libgtk-3-dev libwebkit2gtk-4.1-dev \
  libappindicator3-dev librsvg2-dev patchelf libsqlite3-dev

# Build
cd frontend && npm install && npm run build && cd ..
cargo tauri build --bundles deb,appimage
```

## Cost Considerations

- **Free tier**: 2,000 minutes/month for private repos (unlimited for public)
- **x86_64 builds**: ~10-15 minutes per build
- **ARM64 builds**: ~15-20 minutes per build (if available)
- **Total per release**: ~30-35 minutes for both architectures

For a typical release workflow with 2 architectures, you'll use ~30 minutes per release.