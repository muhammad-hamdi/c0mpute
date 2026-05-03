#!/usr/bin/env sh
# Bump every version pin across the monorepo.
#
# Usage: scripts/bump-version.sh <new-version>
#
#   scripts/bump-version.sh 0.1.1
#   scripts/bump-version.sh 0.2.0
#
# Touches:
#   - root package.json
#   - apps/<app>/package.json (workspace packages)
#   - packages/<pkg>/package.json
#   - root Cargo.toml [workspace.package].version (Rust crates inherit
#     this via `version.workspace = true`)
#   - plugins/<id>/module.toml manifest versions
#
# Does NOT git-tag — run that yourself once happy:
#   git commit -am "bump v$NEW" && git tag v$NEW && git push --tags
set -eu

NEW="${1:-}"
if [ -z "$NEW" ]; then
  echo "usage: $0 <new-version>" >&2
  exit 2
fi

if ! printf '%s' "$NEW" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.+-]*)?$'; then
  echo "✗ '$NEW' doesn't look like a semver" >&2
  exit 2
fi

cd "$(dirname "$0")/.."

# JSON package.json files
find . \
  -name 'package.json' \
  -not -path '*/node_modules/*' \
  -not -path '*/.next/*' \
  -not -path '*/target/*' \
  -print0 |
while IFS= read -r -d '' f; do
  # First "version": "<x.y.z>" only — don't touch deps
  python3 -c "
import json, sys
p = '$f'
with open(p) as fh:
    d = json.load(fh)
if 'version' in d:
    d['version'] = '$NEW'
    with open(p, 'w') as fh:
        json.dump(d, fh, indent=2)
        fh.write('\n')
    print('  updated', p)
" || true
done

# Cargo workspace
python3 - "$NEW" <<'PY'
import sys, re, pathlib
new = sys.argv[1]
p = pathlib.Path('Cargo.toml')
text = p.read_text()
text = re.sub(
    r'(\[workspace\.package\][^\[]*?version\s*=\s*")[^"]+(")',
    r'\g<1>' + new + r'\g<2>',
    text, count=1, flags=re.S,
)
p.write_text(text)
print('  updated Cargo.toml')
PY

# Plugin manifests
for f in plugins/*/module.toml; do
  python3 -c "
import re, pathlib
p = pathlib.Path('$f')
text = p.read_text()
text = re.sub(r'(version\s*=\s*\")[^\"]+(\")', r'\g<1>$NEW\g<2>', text, count=1)
p.write_text(text)
print('  updated', '$f')
"
done

echo
echo "→ bumped to $NEW"
echo "next: git diff && git commit -am \"bump v$NEW\" && git tag v$NEW && git push origin master --tags"
