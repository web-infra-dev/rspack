import type { Compiler, Stats } from '@rspack/core';
import { setTimeout } from 'node:timers/promises';

function runCompiler(compiler: Compiler): Promise<Stats> {
  return new Promise<Stats>((resolve, reject) => {
    compiler.run((err, stats) => {
      if (err) return reject(err);
      if (!stats) return reject(new Error('Compiler returned no stats'));
      resolve(stats);
    });
  });
}

function closeCompiler(compiler: Compiler): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    compiler.close((err) => {
      if (err) return reject(err);
      resolve();
    });
  });
}

async function forceGC(rounds = 1, delayMs = 0) {
  if (typeof global.gc !== 'function') {
    throw new Error(
      'global.gc is unavailable; run this script with --expose-gc',
    );
  }

  for (let i = 0; i < rounds; i++) {
    global.gc();
    await setTimeout(delayMs);
  }
}

function createGCTracker() {
  const finalized = new Set<string>();
  const registry = new FinalizationRegistry<string>((label) => {
    finalized.add(label);
  });

  return {
    track(target: object, label: string) {
      registry.register(target, label);
    },
    async waitForCollection(label: string, rounds = 300) {
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

export { closeCompiler, createGCTracker, forceGC, runCompiler };
