const isWindows = process.platform === 'win32';

const entry = `it("should generate valid code", async () => {${
  isWindows
    ? `expect("skip windows").toBe("skip windows");`
    : `const { staticA, dynamicA } = await import("./entry.mjs"); expect(staticA.a).toBe(1); expect(dynamicA.a).toBe(1);`
}});`;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: `data:text/javascript,${entry}`,
  plugins: [
    function skipWindows(compiler) {
      // windows' path can't include *
      if (!isWindows) {
        const fs = require('fs');
        const path = require('path');
        const dir = path.resolve(__dirname, 'star*');
        const file = path.resolve(dir, 'a.js');

        // STAR-DIAG: temporary instrumentation for the flaky wasm-only resolve
        // failure. Logs the live on-disk state around compile so we can tell
        // whether the file was actually present when the resolver said
        // "Can't resolve" (-> wasm fs/resolver bug) or already gone (-> a
        // create/remove race in this fixture).
        const T0 = Date.now();
        const log = (label, obj) =>
          console.error(
            `[STAR-DIAG-JS] +${Date.now() - T0}ms pid=${process.pid} ${label} ${obj ? JSON.stringify(obj) : ''}`,
          );

        if (!fs.existsSync(dir)) {
          fs.mkdirSync(dir);
        }
        fs.writeFileSync(file, 'export const a = 1;');
        log('created', {
          dirExists: fs.existsSync(dir),
          fileExists: fs.existsSync(file),
        });

        let runCount = 0;
        compiler.hooks.afterCompile.tap('skipWindows-diag', (compilation) => {
          runCount++;
          const errs = compilation.errors || [];
          const starErr = errs.find((e) =>
            String((e && e.message) || e).includes('star*'),
          );
          let dirEntries = null;
          let starDirEntries = null;
          try {
            dirEntries = fs.readdirSync(__dirname);
          } catch (e) {
            dirEntries = `READDIR_ERR:${e}`;
          }
          try {
            starDirEntries = fs.readdirSync(dir);
          } catch (e) {
            starDirEntries = `READDIR_ERR:${e}`;
          }
          log('afterCompile', {
            runCount,
            errorCount: errs.length,
            hasStarError: Boolean(starErr),
            dirExists: fs.existsSync(dir),
            fileExists: fs.existsSync(file),
            dirEntries,
            starDirEntries,
          });
          if (starErr) {
            log('STAR-ERROR', String((starErr && starErr.message) || starErr));
          }
        });

        // cleanup
        compiler.hooks.done.tap('skipWindows', () => {
          log('done -> cleanup', { dirExists: fs.existsSync(dir) });
          fs.rmSync(dir, { recursive: true, force: true });
        });
      }
    },
  ],
};
