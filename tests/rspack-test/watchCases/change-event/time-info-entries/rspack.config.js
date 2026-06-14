const fs = require('node:fs');
const path = require('node:path');

// `aggregateTimeout` is far smaller than `SETTLE_MS`: any rebuild wrongly
// triggered before the file write would complete before the watch step-1
// listener is ready, so that spurious event is not what we test.
const SETTLE_MS = 1500;

class TimeInfoProbe {
  constructor() {
    this.builds = 0;
  }

  apply(compiler) {
    const triggerFile = path.join(compiler.context, 'ctx', 'keep.js');
    const probeFile = path.join(compiler.options.output.path, 'probe.json');
    const dump = (map) => (map ? Array.from(map.entries()) : null);

    compiler.hooks.done.tap('TimeInfoProbe', () => {
      this.builds += 1;
      fs.mkdirSync(path.dirname(probeFile), { recursive: true });
      fs.writeFileSync(
        probeFile,
        JSON.stringify({
          file: dump(compiler.fileTimestamps),
          context: dump(compiler.contextTimestamps),
        }),
      );
      // After the initial build, schedule a file change to trigger a rebuild.
      // The delay ensures the step-1 listener is already registered when the
      // watcher fires, avoiding a race against watchpack's own initial scan.
      if (this.builds === 1) {
        setTimeout(() => {
          fs.writeFileSync(triggerFile, `module.exports = ${Date.now()};\n`);
        }, SETTLE_MS);
      }
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  watchOptions: { aggregateTimeout: 200 },
  plugins: [new TimeInfoProbe()],
};
