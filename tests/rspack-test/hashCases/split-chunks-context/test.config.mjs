import { readFileSync } from 'node:fs';
import path from 'node:path';

/** @type {import('@rspack/test-tools').THashCaseConfig} */
export default {
  validate(stats) {
    const outputs = stats.stats.map((child) => {
      expect(child.hasErrors()).toBe(false);
      const entry = child.compilation.entrypoints.get('main');
      const childChunkCounts = entry.childrenIterable.map((group) => group.chunks.length);
      expect(childChunkCounts).toEqual([3]);
      const [filename] = entry.getFiles();
      const source = readFileSync(
        path.join(child.compilation.outputOptions.path, filename),
        'utf8',
      );
      return { filename, source };
    });
    for (const output of outputs.slice(1)) {
      expect(output).toEqual(outputs[0]);
    }
  },
};
