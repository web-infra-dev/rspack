// Preloaded via NODE_OPTIONS=--require for the faithful `rspack serve` repro.
// Patches the bundled watchpack so we can log the EXACT file/dir path strings
// handed to the watcher on every re-arm cycle, without touching the repro's
// own config or rspack source.
const path = require('node:path');

try {
  const coreDir = path.dirname(require.resolve('@rspack/core/package.json'));
  const wpPath = path.join(coreDir, 'compiled', 'watchpack', 'index.js');
  const Watchpack = require(wpPath);
  if (Watchpack && Watchpack.prototype && typeof Watchpack.prototype.watch === 'function') {
    const real = Watchpack.prototype.watch;
    let cycle = 0;
    Watchpack.prototype.watch = function (a1, a2) {
      cycle += 1;
      let files = [];
      let dirs = [];
      if (a2) {
        files = [...a1];
        dirs = [...a2];
      } else if (a1) {
        files = [...(a1.files || [])];
        dirs = [...(a1.directories || [])];
      }
      const src = files.filter((f) => /[\\/]src[\\/]/.test(f)).sort();
      const srcDirs = dirs.filter((f) => /[\\/]src([\\/]|$)/.test(f)).sort();
      console.error(`[PRELOAD watchpack re-arm #${cycle}] files=${files.length} dirs=${dirs.length}`);
      console.error('  src files:', JSON.stringify(src));
      console.error('  src dirs :', JSON.stringify(srcDirs));
      console.error('  App.tsx present:', files.some((f) => /App\.tsx$/.test(f)));
      return real.apply(this, arguments);
    };
    console.error(`[PRELOAD] patched watchpack at ${wpPath}`);
  } else {
    console.error(`[PRELOAD] watchpack.prototype.watch missing at ${wpPath}`);
  }
} catch (e) {
  console.error('[PRELOAD] failed to patch watchpack:', e && e.message);
}
