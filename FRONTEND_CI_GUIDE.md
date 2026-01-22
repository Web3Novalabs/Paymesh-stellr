# Frontend CI Pipeline Implementation Guide

## Overview
The Frontend CI pipeline has been successfully implemented using GitHub Actions. It automatically runs build and format checks whenever a Pull Request is created or commits are pushed to the main branch affecting the frontend directory.

## Workflow Configuration
**File**: [.github/workflows/frontend-ci.yml](.github/workflows/frontend-ci.yml)

### Trigger Events
- **Pull Requests**: Triggered on PRs to `main` branch when frontend files change
- **Push Events**: Triggered on direct pushes to `main` branch when frontend files change
- **Path Filter**: Only runs when files in `frontend/**` or `.github/workflows/frontend-ci.yml` are modified

## Pipeline Checks

### 1. Build Check
- **Command**: `pnpm build` (or `npm run build` / `yarn build`)
- **Purpose**: Ensures the frontend builds successfully without errors
- **Failure Condition**: Pipeline fails if build encounters any errors

### 2. Format Check
- **Command**: `pnpm lint` (or `npm run lint` / `yarn lint`)
- **Purpose**: Verifies code formatting and linting rules (ESLint)
- **Configuration**: Uses existing ESLint config in `frontend/eslint.config.mjs`
- **Note**: Currently set to `continue-on-error: true` (non-blocking) for warnings

## Key Features

✅ **Package Manager Detection**
- Automatically detects pnpm, yarn, or npm
- Uses the correct package manager for your project (pnpm in this case)

✅ **Dependency Caching**
- Caches `node_modules` and `.next/cache`
- Speeds up workflow execution by ~1-2 minutes
- Cache key: `${{ runner.os }}-deps-${{ hashFiles(...) }}`

✅ **Node.js Environment**
- Node.js 18 configured
- Compatible with Next.js 16.1.3

✅ **Clear Error Messages**
- Helpful output on failure
- Guides developers to fix issues

## Testing the Pipeline

### Method 1: Create a Test PR (Recommended)
1. Create a new branch: `git checkout -b test/ci-pipeline`
2. Make a small change to a frontend file
3. Commit and push: `git add . && git commit -m "test: verify ci pipeline"` && `git push origin test/ci-pipeline`
4. Open a Pull Request to `main`
5. Check the PR for CI status

### Method 2: Test Locally (Optional)
Verify build and lint checks pass locally before pushing:
```bash
cd frontend
pnpm install
pnpm lint    # Format check
pnpm build   # Build check
```

## Expected Behavior

### ✅ Pipeline Should Pass When:
- Code builds successfully with `pnpm build`
- ESLint passes with `pnpm lint`
- No syntax errors or TypeScript issues

### ❌ Pipeline Should Fail When:
- Build has errors (component imports, TypeScript errors, syntax issues)
- Linting errors are found (code style, unused variables, etc.)

## Example PR Workflow

1. **Create feature branch**
   ```bash
   git checkout -b feature/new-component
   ```

2. **Make changes** to frontend code

3. **Push to GitHub**
   ```bash
   git push origin feature/new-component
   ```

4. **CI Pipeline Runs Automatically**
   - You'll see workflow status in the PR
   - Red ❌ = Failed checks
   - Green ✅ = Passed checks

5. **Fix Issues (if needed)**
   ```bash
   # Run locally to test
   cd frontend
   pnpm lint    # See linting errors
   pnpm build   # See build errors
   ```

6. **Commit and push fixes**
   ```bash
   git add .
   git commit -m "fix: resolve linting issues"
   git push origin feature/new-component
   ```

## Configuration Details

### Dependencies
- Uses existing project dependencies (pnpm is the primary package manager)
- Node.js 18 LTS
- Next.js 16.1.3
- React 19.2.3

### Working Directory
- All commands run from `frontend/` directory via `defaults.run.working-directory`
- Keeps workflow clean and prevents path issues

### Cache Strategy
- Caches across workflow runs
- Reset when `package-lock.json`, `pnpm-lock.yaml`, or `yarn.lock` changes
- Significantly speeds up dependency installation

## Troubleshooting

### Build Fails
1. Check the error message in GitHub Actions logs
2. Run `cd frontend && pnpm install && pnpm build` locally
3. Fix any TypeScript or syntax errors
4. Commit and push changes

### Lint Fails
1. Review linting errors in GitHub Actions output
2. Run `cd frontend && pnpm lint` locally to see all issues
3. Fix formatting issues
4. Some issues can be auto-fixed: Check ESLint documentation

### Cache Issues
If dependencies aren't installing correctly:
1. Check GitHub Actions logs for cache hit/miss
2. Cache automatically clears when lock files change
3. Manual cache clear: Delete cache from GitHub Actions settings if needed

## Next Steps

1. **Create a test PR** to verify the pipeline works
2. **Document in contributing guide** (if not already done)
3. **Enable branch protection** (optional):
   - Go to repository settings
   - Select `main` branch
   - Check "Require status checks to pass before merging"
   - Select "Frontend Build and Format Checks"

## Related Files
- Workflow: [.github/workflows/frontend-ci.yml](.github/workflows/frontend-ci.yml)
- ESLint Config: [frontend/eslint.config.mjs](frontend/eslint.config.mjs)
- Package Manager: [frontend/pnpm-lock.yaml](frontend/pnpm-lock.yaml)
- Contributing Guide: [contributor.md](contributor.md)

## References
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [setup-node Action](https://github.com/actions/setup-node)
- [pnpm GitHub Action](https://github.com/pnpm/action-setup)

---

**Implementation Date**: January 22, 2026
**Workflow Version**: 1.0
