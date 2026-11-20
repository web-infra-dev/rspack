import path from 'node:path';

/** @type {import('@rspack/test-tools').THashCaseConfig} */
export default {
  validate(stats) {
    const [first, second] = stats.stats.map((child) => {
      expect(child.hasErrors()).toBe(false);
      const { compilation } = child;
      const anonymous = [...compilation.chunks].filter((chunk) => !chunk.name);
      expect(anonymous).toHaveLength(1);
      const [chunk] = anonymous;
      const modules = Array.from(
        compilation.chunkGraph.getChunkModulesIterable(chunk),
        (module) => path.basename(module.nameForCondition()),
      ).sort();
      expect(modules).toContain('b.js');
      return { modules, files: [...chunk.files].sort() };
    });
    expect(second).toEqual(first);
  },
};
