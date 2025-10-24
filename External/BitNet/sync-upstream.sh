#!/bin/bash
# ============================================================================
# BitNet Upstream Sync Script
# ============================================================================
# Syncs with upstream microsoft/BitNet to get latest changes
# ============================================================================

set -e  # Exit on error

echo ""
echo "============================================================================"
echo "Syncing with upstream microsoft/BitNet"
echo "============================================================================"
echo ""

# Get script directory (BitNet root)
BITNET_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$BITNET_ROOT"

# Check current branch
CURRENT_BRANCH=$(git branch --show-current)
echo "Current branch: $CURRENT_BRANCH"
echo ""

# Fetch upstream
echo "Fetching upstream changes..."
git fetch upstream

# Get the default branch name from upstream
UPSTREAM_DEFAULT=$(git remote show upstream | grep 'HEAD branch' | cut -d' ' -f5)
echo "Upstream default branch: $UPSTREAM_DEFAULT"
echo ""

# Merge upstream main/master into current branch
echo "Merging upstream/$UPSTREAM_DEFAULT into $CURRENT_BRANCH..."
git merge upstream/$UPSTREAM_DEFAULT --no-edit

echo ""
echo "============================================================================"
echo "✅ Sync Complete!"
echo "============================================================================"
echo ""
echo "Latest upstream changes merged into: $CURRENT_BRANCH"
echo ""
echo "Next steps:"
echo "  1. Review changes: git log --oneline -10"
echo "  2. Run builds: ./build-all-linux.sh (or build-all-*.sh/bat for your platform)"
echo ""

