import fs from 'node:fs';
import path from 'node:path';

const clientSource = (name, comment = true) => `
export async function load() {
  const [, { ${name} }] = await Promise.all([
    Promise.resolve(null),
    import(${comment ? `/* webpackExports: '${name}' */ ` : ''}'./lib.js'),
  ]);
  return ${name};
}
`;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [false, true].map(ignoreWarnings => {
  let root;
  let expectedName = 'Foo';
  let previousMessage;
  let builds = 0;

  return {
    description: `should preserve diagnostic rendering across rebuilds (ignoreWarnings: ${ignoreWarnings})`,
    options(context) {
      root = context.getDist(`cached-diagnostics-${ignoreWarnings}`);
      fs.mkdirSync(root, { recursive: true });
      fs.writeFileSync(path.join(root, 'index.js'), "import './client.js';\n");
      fs.writeFileSync(path.join(root, 'client.js'), clientSource('Foo'));
      fs.writeFileSync(path.join(root, 'lib.js'), 'export const Foo = 1, Bar = 2;\n');
      return {
        mode: 'development',
        context: root,
        entry: './index.js',
        devtool: false,
        incremental: true,
        ignoreWarnings: ignoreWarnings ? [/Useless magic comments/] : [],
      };
    },
    compiler(_context, compiler) {
      compiler.hooks.done.tap('CachedDiagnosticsTest', stats => {
        const { compilation } = stats;
        expect(stats.hasErrors()).toBe(false);
        const hasWarning = expectedName !== undefined;
        expect(compilation.warnings).toHaveLength(hasWarning ? 1 : 0);
        if (hasWarning) {
          const message = compilation.warnings[0].message;
          expect(message).toContain('Useless magic comments');
          expect(message).toContain(`webpackExports: '${expectedName}'`);
          if (builds === 1) expect(message).toBe(previousMessage);
          if (builds === 2) expect(message).not.toBe(previousMessage);
          previousMessage = message;
        }

        const warnings = compilation.getWarnings();
        const count = hasWarning && !ignoreWarnings ? 1 : 0;
        expect(warnings).toHaveLength(count);
        if (count) {
          expect(warnings[0].message).toBe(previousMessage);
          warnings[0].message = 'modified by a plugin';
          expect(compilation.getWarnings()[0].message).toBe(previousMessage);
        }

        for (const colors of [false, true, false]) {
          const json = stats.toJson({ all: false, warnings: true, colors });
          expect(json.warnings).toHaveLength(count);
          if (count) {
            const message = json.warnings[0].message;
            expect(message).toContain(`webpackExports: '${expectedName}'`);
            expect(message.includes('\u001b[')).toBe(colors);
            if (!colors) expect(message).toBe(previousMessage);
          }
        }
        builds++;
      });
    },
    async build(_context, compiler) {
      const run = modifiedFiles => new Promise((resolve, reject) => {
        compiler.run(error => error ? reject(error) : resolve(), { modifiedFiles });
      });
      await run();

      const entry = path.join(root, 'index.js');
      fs.appendFileSync(entry, '// entry changed\n');
      await run(new Set([entry]));

      const client = path.join(root, 'client.js');
      expectedName = 'Bar';
      fs.writeFileSync(client, clientSource('Bar'));
      await run(new Set([client]));

      expectedName = undefined;
      fs.writeFileSync(client, clientSource('Bar', false));
      await run(new Set([client]));
    },
    check() {
      expect(builds).toBe(4);
    },
  };
});
