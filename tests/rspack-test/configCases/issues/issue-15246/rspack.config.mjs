import path from 'node:path';

const PLUGIN_NAME = 'Issue15246Plugin';

class Issue15246Plugin {
  /**
   * @param {import('@rspack/core').Compiler} compiler
   */
  apply(compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.afterProcessAssets.tap(PLUGIN_NAME, () => {
        const chunklessModule = Array.from(compilation.modules).find(
          (module) =>
            module.resource ===
            path.join(import.meta.dirname, 'node_modules/components/index.js'),
        );

        expect(chunklessModule).toBeDefined();
        expect(compilation.chunkGraph.getModuleChunks(chunklessModule)).toEqual(
          [],
        );

        expect(() =>
          compilation.codeGenerationResults.get(chunklessModule, undefined),
        ).toThrow('No code generation entry');
      });
    });
  }
}

/** @type {import('@rspack/core').Configuration} */
export default {
  mode: 'production',
  target: 'node',
  entry: {
    main: {
      import: './index.js',
      layer: 'miniprogram',
    },
  },
  optimization: {
    concatenateModules: false,
    minimize: false,
  },
  module: {
    rules: [
      {
        test: /\.less$/,
        type: 'css/auto',
        use: ['less-loader'],
      },
    ],
    parser: {
      'css/auto': {
        namedExports: false,
      },
    },
  },
  plugins: [new Issue15246Plugin()],
};
