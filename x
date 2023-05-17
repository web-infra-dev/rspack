#!/usr/bin/env node
// ^ This cannot be `#!/usr/bin/env zx` because `zx` will create a temporary `.mjs` file to execute on if this is extensionless.

import('./x.mjs').catch((e) => {
  console.error('error:', e);
  process.exit(1)
}); // Using dynamic import because ESM cannot be extensionless.

// NOTE:
// Add this directory to your $PATH for executing without dot slash (`./x`).
// In your ~/.bashrc or ~/.zshrc, add:
// export PATH="/path/to/rspack:$PATH"
