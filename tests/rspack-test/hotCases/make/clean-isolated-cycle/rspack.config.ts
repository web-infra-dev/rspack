import { defineConfig, definePlugin } from '@rspack/cli';
import { type StatsModule } from '@rspack/core';

function check_issuer(modules: StatsModule[]) {
  let available: string[] = [];
  let map: Record<string, string | true> = {};
  for (const m of modules) {
    map[m.identifier!] = m.issuer || true;
  }
  for (const m of modules) {
    if (available.includes(m.identifier!)) {
      continue;
    }
    let paths: string[] = [];
    let current = m.identifier!;
    while (true) {
      if (paths.includes(current)) {
        throw new Error('has cycle issuer');
      }
      paths.push(current);
      const next = map[current];
      if (next == undefined) {
        throw new Error('current issuer module not exist');
      }
      if (next == true) {
        available = available.concat(paths);
        break;
      }
      current = next;
    }
  }
}

function assert_cycle_module_count(modules: StatsModule[], count: number) {
  let cycle1_count = 0;
  let cycle2_count = 0;
  for (const m of modules) {
    if (/[\/\\]cycle1[\/\\]/.test(m.identifier!)) {
      cycle1_count++;
    }
    if (/[\/\\]cycle2[\/\\]/.test(m.identifier!)) {
      cycle2_count++;
    }
  }
  if (cycle1_count !== count) {
    throw new Error('cycle1 count eq error');
  }
  if (cycle2_count !== count) {
    throw new Error('cycle2 count eq error');
  }
}

export default defineConfig({
  context: import.meta.dirname,
  plugins: [
    definePlugin({
      apply(compiler) {
        let time = 0;
        compiler.hooks.done.tap('PLUGIN', (stats) => {
          let { modules } = stats.toJson({ modules: true });
          if (time == 0) {
            assert_cycle_module_count(modules!, 3);
          }
          if (time == 1) {
            assert_cycle_module_count(modules!, 3);
          }
          if (time == 2) {
            assert_cycle_module_count(modules!, 3);
          }
          if (time == 3) {
            assert_cycle_module_count(modules!, 0);
          }
          check_issuer(modules!);
          time++;
        });
      },
    }),
  ],
});
