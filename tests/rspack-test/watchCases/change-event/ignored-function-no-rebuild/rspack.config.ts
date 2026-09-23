import { defineConfig } from '@rspack/cli';
import { type Compiler } from '@rspack/core';
import fs from 'node:fs';
import path from 'node:path';
// `aggregateTimeout` is far smaller than `SETTLE_MS`: a rebuild wrongly
// triggered by the ignored dependency would land well before `trigger.js` is
// touched, bumping the build count the probe records.
const SETTLE_MS = 1500;

class IgnoredRebuildProbe {
  builds = 0;

  apply(compiler: Compiler) {
    const ignoredDep = path.join(compiler.context, '__ignored__', 'dep.js');
    const triggerFile = path.join(compiler.context, 'trigger.js');
    const probeFile = path.join(compiler.options.output.path!, 'probe.json');

    compiler.hooks.done.tap(IgnoredRebuildProbe.name, () => {
      this.builds += 1;
      fs.mkdirSync(path.dirname(probeFile), { recursive: true });
      fs.writeFileSync(probeFile, JSON.stringify({ builds: this.builds }));
      if (this.builds === 1) {
        fs.writeFileSync(ignoredDep, 'module.exports = "changed";');
        setTimeout(() => {
          fs.writeFileSync(triggerFile, 'module.exports = "changed";');
        }, SETTLE_MS);
      }
    });
  }
}

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  watchOptions: {
    aggregateTimeout: 200,
    // The arbitrary function is the one form that receives the entry with the
    // platform separators, and it decides for every entry on its own — so it
    // has to cover the whole subtree itself.
    ignored: (entry) => entry.replace(/\\/g, '/').includes('/__ignored__/'),
  },
  plugins: [new IgnoredRebuildProbe()],
});
