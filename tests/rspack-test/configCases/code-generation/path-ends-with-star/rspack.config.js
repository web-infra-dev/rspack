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
    function createStarDir(compiler) {
      // windows' path can't include *
      if (!isWindows) {
        const fs = require('fs');
        const path = require('path');
        const dir = path.resolve(__dirname, 'star*');
        const file = path.resolve(dir, 'a.js');
        // This case is run in parallel by both Config.* and RuntimeModeConfig.*
        // against this shared source dir. Create the `*` fixture idempotently and
        // never delete it mid-run: a done-hook rmSync would remove `star*` while
        // the other suite is still building it, producing a flaky
        // "Can't resolve ./star*/a.js" (only seen on the slower wasm target).
        fs.mkdirSync(dir, { recursive: true });
        if (!fs.existsSync(file)) {
          fs.writeFileSync(file, 'export const a = 1;');
        }
      }
    },
  ],
};
