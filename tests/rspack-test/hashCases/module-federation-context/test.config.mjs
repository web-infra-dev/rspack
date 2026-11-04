import fs from 'node:fs';
import path from 'node:path';

/** @type {import('@rspack/test-tools').THashCaseConfig} */
export default {
  validate(stats) {
    expect(stats.hasErrors()).toBe(false);
    expect(stats.stats).toHaveLength(4);

    for (let index = 0; index < stats.stats.length; index += 2) {
      const [a, b] = stats.stats.slice(index, index + 2);
      const moduleIds = (child) =>
        child
          .toJson({ all: false, modules: true, ids: true })
          .modules.map((module) => module.id)
          .sort();
      const chunkIds = (child) =>
        child
          .toJson({ all: false, chunks: true, ids: true })
          .chunks.map((chunk) => chunk.id)
          .sort();

      expect(moduleIds(a)).toEqual(moduleIds(b));
      expect(chunkIds(a)).toEqual(chunkIds(b));

      const assets = (child) =>
        child
          .toJson({ all: false, assets: true })
          .assets.map((asset) => asset.name)
          .sort();
      expect(assets(a)).toEqual(assets(b));
      for (const asset of assets(a)) {
        expect(
          fs.readFileSync(path.join(a.compilation.outputOptions.path, asset)),
        ).toEqual(fs.readFileSync(path.join(b.compilation.outputOptions.path, asset)));
      }

      const sharedIdentifiers = (child) =>
        child
          .toJson({ all: false, modules: true })
          .modules.map((module) => module.identifier)
          .filter((identifier) => /^(consume|provide) shared module/.test(identifier));
      const checkSharedIdentifiers = (child) => {
        const context = child.compilation.options.context;
        const identifiers = sharedIdentifiers(child);
        expect(identifiers).toContain(
          `consume shared module (default) shared-lib@^1.0.0 (strict) (fallback: ${path.join(context, 'shared.js')})`,
        );
        expect(identifiers).toContain(
          'provide shared module (default) shared-lib@1.0.0 = ./shared.js',
        );
      };
      checkSharedIdentifiers(a);
      checkSharedIdentifiers(b);
    }
  },
};
