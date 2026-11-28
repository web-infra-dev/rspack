import { join } from 'node:path';

export default {
  validate(stats) {
    const origin = `parallel-loader|${join(import.meta.dirname, 'resource.js')}`;
    const { compilation } = stats.stats[0];
    const buildDependency = join(import.meta.dirname, 'dependency.js');
    expect(compilation.buildDependencies.has(buildDependency)).toBe(true);
    const entries = compilation.logging.get(origin);
    const timers = entries.filter((entry) => entry.type === 'time');
    expect(timers.map((entry) => entry.args[0])).toEqual(['work', 'work']);
  },
};
