import { readFileSync } from 'node:fs';
import path from 'node:path';

/** @type {import('@rspack/test-tools').THashCaseConfig} */
export default {
  validate(stats) {
    const outputs = stats.stats.map((child) => {
      expect(child.hasErrors()).toBe(false);
      const [filename] = child.compilation.entrypoints.get('main').getFiles();
      const source = readFileSync(
        path.join(child.compilation.outputOptions.path, filename),
        'utf8',
      );
      expect(source.match(/__webpack_require__\.e\(\d+\)/g)).toHaveLength(3);
      return { filename, source };
    });
    for (const output of outputs.slice(1)) {
      expect(output).toEqual(outputs[0]);
    }
  },
};
