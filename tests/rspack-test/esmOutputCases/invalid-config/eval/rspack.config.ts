import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

const previousDevtool = 'eval';
const fallbackDevtool = false;
const expectedWarning = `\`devtool: "${previousDevtool}"\` is not supported with \`library.type: "modern-module"\` because its modules are rendered as top-level code. Rspack has changed \`devtool\` to \`${fallbackDevtool}\`.`;

class AssertEvalFallbackPlugin {
  apply(compiler: Compiler) {
    let warning: unknown;

    compiler.hooks.infrastructureLog.tap(
      'AssertEvalFallbackPlugin',
      (name, type, args) => {
        if (name === 'rspack.RspackOptionsApply' && type === 'warn') {
          warning = args[0];
          return true;
        }
      },
    );

    compiler.hooks.afterPlugins.tap('AssertEvalFallbackPlugin', () => {
      expect(compiler.options.devtool).toBe(fallbackDevtool);
      expect(warning).toBe(expectedWarning);
    });
  }
}

export default defineConfig({
  devtool: previousDevtool,
  plugins: [new AssertEvalFallbackPlugin()],
});
