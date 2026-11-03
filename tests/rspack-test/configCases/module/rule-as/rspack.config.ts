import { defineConfig } from '@rspack/cli';
import type { Compiler, NormalModule } from '@rspack/core';

class CheckMatchResourcePlugin {
  apply(compiler: Compiler) {
    compiler.hooks.finishMake.tap('CheckMatchResourcePlugin', (compilation) => {
      const modules = [...compilation.modules].filter(
        (module): module is NormalModule => 'resource' in module,
      );
      const findByResource = (resource: string) =>
        modules.find((module) => module.resource.endsWith(resource));

      expect(findByResource('explicit.js')?.matchResource).toBe('virtual.txt');
      expect(findByResource('rematch.js')?.matchResource).toBe('virtual.txt');
      expect(findByResource('priority.custom')?.matchResource).toBe('inline');
    });
  }
}

export default defineConfig({
  module: {
    rules: [
      {
        test: /explicit\.js$/,
        as: 'virtual.txt.rspack[asset/source]',
      },
      {
        test: /rematch\.js$/,
        as: 'virtual.txt',
      },
      {
        test: /\.txt$/,
        type: 'asset/source',
      },
      {
        test: /priority\.custom$/,
        as: 'rule.txt.rspack[asset/source]',
      },
    ],
  },
  plugins: [new CheckMatchResourcePlugin()],
});
