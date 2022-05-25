import { promises as fs } from 'fs';
import * as path from 'path';

import {
  expect,
  SourceMapConsumer,
  convertSourceMap,
} from '@rspack/test-toolkit';

import { Rspack, RspackPlugin } from '../src/node/rspack';

const fixtureRoot = path.resolve(__dirname, './fixtures');

describe('rspack', () => {
  it('should work with plugins', async () => {
    const fixture = path.join(fixtureRoot, 'plugin');

    const plugin: RspackPlugin = {
      async onLoad(context) {
        if (context.id.endsWith('foo.js')) {
          return {
            content: `${await fs.readFile(
              context.id,
              'utf-8'
            )}console.log("fooo");`,
            loader: 'js',
          };
        }
      },
      async onResolve(context) {
        if (context.importee === 'foo') {
          return {
            uri: path.join(fixture, 'foo.js'),
            external: false,
          };
        }
      },
    };

    const rspack = new Rspack({
      entries: { index: path.join(fixture, 'index.js') },
      plugins: [plugin],
      output: {
        sourceMap: false,
      },
    });

    await rspack.build();
    expect(
      await fs.readFile(path.join(fixture, 'dist', 'index.js'), 'utf-8')
    ).toMatchSnapshot();
  });

  it('should generate correct source map', async () => {
    const fixture = path.join(fixtureRoot, 'source-map');

    const rspack = new Rspack({
      entries: { index: path.join(fixture, 'index.js') },
      output: {
        outdir: path.join(fixture, 'dist'),
        sourceMap: true,
      },
    });

    await rspack.build();
    const code = await fs.readFile(
      path.join(fixture, 'dist', 'index.js'),
      'utf-8'
    );
    // TODO: use `rspack-sources://`, ref: https://webpack.js.org/configuration/output/#outputdevtoolmodulefilenametemplate
    // expect(code).toMatchSnapshot();

    const sourceMap = convertSourceMap.fromSource(code);
    const consumer = await new SourceMapConsumer(sourceMap.toObject());

    const meta1 = consumer.originalPositionFor({
      line: 3,
      column: 4,
    });
    expect(meta1.line).to.eq(1);
    expect(meta1.column).to.eq(0);
    expect(meta1.source.includes('index.js')).to.be.true;

    const meta2 = consumer.originalPositionFor({
      line: 4,
      column: 4,
    });
    expect(meta2.line).to.eq(2);
    expect(meta2.column).to.eq(4);
    expect(meta2.source.includes('index.js')).to.be.true;
  });
});
