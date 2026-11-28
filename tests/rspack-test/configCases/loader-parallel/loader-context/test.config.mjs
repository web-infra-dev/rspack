import { join } from 'node:path';

export default {
  validate(stats) {
    const origin = `parallel-loader|${join(import.meta.dirname, 'resource.js')}`;
    const entries = stats.stats[0].compilation.logging.get(origin);
    const timers = entries.filter((entry) => entry.type === 'time');
    expect(timers.map((entry) => entry.args[0])).toEqual(['work', 'work']);
  },
};
