import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { CssExtractRspackPlugin } from '@rspack/core';

const require = createRequire(import.meta.url);
const roots = Array.from({ length: 8 }, (_, index) => `root${index}.css`);
const built = new Set();
const rootStyle = (index, color, shared = true) =>
  `${shared ? '@import "./shared.css";' : ''}.root${index} { color: ${color}; }`;

export default {
  description: 'CSS watch updates preserve locality with shared imports',
  options(context) {
    const input = context.getDist('input');
    fs.mkdirSync(input, { recursive: true });
    fs.writeFileSync(
      path.join(input, 'entry.js'),
      roots.map((root) => `import './${root}';`).join('\n'),
    );
    fs.writeFileSync(
      path.join(input, 'shared.css'),
      '.shared { color: green; }',
    );
    for (const [index, root] of roots.entries()) {
      fs.writeFileSync(path.join(input, root), rootStyle(index, 'red'));
    }
    return {
      context: input,
      mode: 'development',
      cache: true,
      incremental: true,
      devtool: false,
      entry: './entry.js',
      output: { path: context.getDist('output'), filename: 'main.js' },
      module: {
        rules: [
          {
            test: /\.css$/,
            type: 'javascript/auto',
            use: [CssExtractRspackPlugin.loader, require.resolve('css-loader')],
          },
        ],
      },
      plugins: [new CssExtractRspackPlugin({ filename: 'main.css' })],
    };
  },
  async compiler(context, compiler) {
    compiler.hooks.thisCompilation.tap('Test', (compilation) => {
      compilation.hooks.buildModule.tap('Test', (module) => {
        if (
          module.type === 'javascript/auto' &&
          roots.includes(path.basename(module.resource || ''))
        ) {
          built.add(path.basename(module.resource));
        }
      });
    });
  },
  async build(context, compiler) {
    const initial = await new Promise((resolve) => compiler.run(resolve));
    expect(initial).toBeNull();
    const update = async (file, source, expectedRoots) => {
      const target = context.getDist(path.join('input', file));
      fs.writeFileSync(target, source);
      built.clear();
      const error = await new Promise((resolve) => {
        compiler.__internal__rebuild(new Set([target]), new Set(), resolve);
      });
      expect(error).toBeFalsy();
      expect(compiler._lastCompilation.errors).toHaveLength(0);
      expect([...built].sort()).toEqual([...expectedRoots].sort());
      return compiler._lastCompilation
        .getAsset('main.css')
        .source.source()
        .toString();
    };

    let css = await update('root0.css', rootStyle(0, 'blue'), ['root0.css']);
    expect(css).toMatch(/\.root0\s*\{\s*color:\s*blue/);
    expect(css).toMatch(/\.shared\s*\{\s*color:\s*green/);

    css = await update('shared.css', '.shared { color: purple; }', roots);
    expect(css).toMatch(/\.shared\s*\{\s*color:\s*purple/);
    expect(css).not.toContain('green');

    css = await update('root0.css', rootStyle(0, 'orange', false), [
      'root0.css',
    ]);
    expect(css).toMatch(/\.root0\s*\{\s*color:\s*orange/);
    expect(css).toMatch(/\.shared\s*\{\s*color:\s*purple/);
  },
};
