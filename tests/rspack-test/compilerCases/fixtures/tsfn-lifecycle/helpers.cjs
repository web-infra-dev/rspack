const { setTimeout } = require("node:timers/promises");

function runCompiler(compiler) {
  return new Promise((resolve, reject) => {
    compiler.run((err, stats) => {
      if (err) return reject(err);
      resolve(stats);
    });
  });
}

function closeCompiler(compiler) {
  return new Promise((resolve, reject) => {
    compiler.close(err => {
      if (err) return reject(err);
      resolve();
    });
  });
}

async function forceGC(rounds = 1, delayMs = 0) {
  if (typeof global.gc !== "function") {
    throw new Error("global.gc is unavailable; run this script with --expose-gc");
  }

  for (let i = 0; i < rounds; i++) {
    global.gc();
    await setTimeout(delayMs);
  }
}

function createGCTracker() {
  const finalized = new Set();
  const registry = new FinalizationRegistry(label => {
    finalized.add(label);
  });

  return {
    track(target, label) {
      registry.register(target, label);
    },
    async waitForCollection(label, rounds = 300) {
      for (let i = 0; i < rounds; i++) {
        await forceGC(1, 5);
        if (finalized.has(label)) {
          return;
        }
      }

      throw new Error(`${label} was not garbage collected`);
    },
  };
}

module.exports = {
  closeCompiler,
  createGCTracker,
  forceGC,
  runCompiler,
};
