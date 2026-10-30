import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

export default ['native-fs', 'input-fs', 'js'].map((mode) => {
  const loader = mode === 'js' ? 'less-loader' : 'builtin:less-loader';
  const store = fs.mkdtempSync(path.join(os.tmpdir(), 'less-pnpm-'));
  const name = path.basename(store);
  const pkg = path.join(store, 'instance/node_modules/package');
  const theme = path.join(store, 'theme');
  const link = path.join(process.cwd(), 'node_modules', name);
  fs.mkdirSync(pkg, { recursive: true });
  fs.mkdirSync(theme);
  fs.writeFileSync(
    path.join(pkg, 'mixins.less'),
    '@import "nested-theme/tokens.less";',
  );
  fs.writeFileSync(
    path.join(theme, 'tokens.less'),
    '@color: purple; .from-theme { color: @color; }',
  );
  fs.symlinkSync(
    theme,
    path.join(path.dirname(pkg), 'nested-theme'),
    'junction',
  );
  fs.symlinkSync(pkg, link, 'junction');
  const realMixin = fs.realpathSync(path.join(pkg, 'mixins.less'));
  const realTokens = fs.realpathSync(path.join(theme, 'tokens.less'));
  let realpathCalls = 0;
  return {
    experiments:
      mode === 'input-fs' ? { useInputFileSystem: [/less-pnpm-/] } : {},
    module: {
      rules: [
        {
          test: /\.less$/,
          type: 'asset/source',
          use: [
            {
              loader: './assert-map-loader.mjs',
              options: { realMixin, realTokens },
            },
            {
              loader,
              options: {
                sourceMap: true,
                additionalData: `@import "${name}/mixins.less";`,
              },
            },
          ],
        },
      ],
    },
    plugins: [
      {
        apply(compiler) {
          if (mode === 'input-fs') {
            compiler.inputFileSystem = {
              ...fs,
              realpath(...args) {
                realpathCalls++;
                return fs.realpath(...args);
              },
            };
          }
          compiler.hooks.afterCompile.tap(
            'LessPnpmDependencies',
            (compilation) => {
              if (compilation.errors.length) return;
              assert(compilation.fileDependencies.has(realMixin));
              assert(compilation.fileDependencies.has(realTokens));
              if (mode === 'input-fs') assert(realpathCalls > 0);
            },
          );
          compiler.hooks.shutdown.tap('LessPnpmCleanup', () => {
            fs.unlinkSync(link);
            fs.rmSync(store, { recursive: true, force: true });
          });
        },
      },
    ],
  };
});
