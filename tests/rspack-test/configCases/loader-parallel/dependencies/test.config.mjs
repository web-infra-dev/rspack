import { join } from 'node:path';

export default {
  findBundle: () => [],
  validate(stats) {
    const buildDependency = join(import.meta.dirname, 'index.js');
    const { compilation } = stats.stats[0];
    expect(compilation.buildDependencies.has(buildDependency)).toBe(true);
  },
};
