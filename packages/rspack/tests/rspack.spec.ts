import { promises as fs } from 'fs';
import * as path from 'path';
import assert from 'assert';

import { expect, SourceMapConsumer, convertSourceMap } from '@rspack/test-toolkit';

import { Rspack, RspackPlugin } from '../src/node/rspack';

const fixtureRoot = path.resolve(__dirname, './fixtures');

describe('rspack', () => {
  it('should work with plugins', async () => {
    const fixture = path.join(fixtureRoot, 'plugin');

    const plugin: RspackPlugin = {
      async load(id) {
        if (id.endsWith('foo.js')) {
          return {
            content: `${await fs.readFile(id, 'utf-8')}console.log("fooo");`,
            loader: 'js',
          };
        }
      },
      async resolve(source) {
        if (source === 'foo') {
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
        outdir: path.join(fixture, 'dist'),
        sourceMap: false,
      },
    });

    await rspack.build();
    expect(await fs.readFile(path.join(fixture, 'dist', 'index.js'), 'utf-8')).toMatchSnapshot();
  });

  it.skip('should generate correct source map', async () => {
    const fixture = path.join(fixtureRoot, 'source-map');

    const rspack = new Rspack({
      entries: { index: path.join(fixture, 'index.js') },
      output: {
        outdir: path.join(fixture, 'dist'),
        sourceMap: true,
      },
    });

    await rspack.build();
    const code = await fs.readFile(path.join(fixture, 'dist', 'index.js'), 'utf-8');
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

  it('should keep the correct order', async () => {
    const fixture = path.join(fixtureRoot, 'basic');

    const order = {
      buildStart: [],
      resolve: [],
      load: [],
      buildEnd: [],
    };

    let resolveRound = 1;
    let loadRound = 1;

    const rspack = new Rspack({
      entries: { index: path.join(fixture, 'index.js') },
      output: {
        outdir: path.join(fixture, 'dist'),
      },
      plugins: [
        {
          name: 'plugin-1',
          async buildStart() {
            return new Promise((res) => {
              setTimeout(() => {
                order.buildStart.push(1);
                res();
              });
            });
          },
          async load() {
            expect(order.buildStart.length).to.eq(2);
            expect(order.buildStart).to.deep.equal([2, 1]);

            if (loadRound === 1) {
              expect(order.load.length).to.eq(0);
            }

            if (loadRound === 2) {
              expect(order.load.length).to.eq(2);
            }

            order.load.push(1);
            loadRound += 1;
          },
          async resolve() {
            if (resolveRound === 1) {
              expect(order.load.length).to.eq(0);
            }

            if (resolveRound === 2) {
              expect(order.load.length).to.eq(2);
            }
            resolveRound += 1;
          },
          async buildEnd() {
            order.buildEnd.push(1);
          },
        },
        {
          name: 'plugin-2',
          async buildStart() {
            order.buildStart.push(2);
          },
          async load() {
            order.load.push(2);

            if (loadRound === 1) {
              expect(order.load.length).to.eq(1);
              expect(order.load[0]).to.eq(1);
            }

            if (loadRound === 2) {
              expect(order.load.length).to.eq(2);
              expect(order.load[0]).to.eq(1);
            }
          },
          async resolve() {
            if (resolveRound === 1) {
              expect(order.resolve.length).to.eq(1);
            }
          },
          async buildEnd() {
            return new Promise((res) => {
              setTimeout(() => {
                order.buildEnd.push(2);
                res();
              });
            });
          },
        },
      ],
    });

    await rspack.build();

    expect(order.buildEnd).to.deep.equal([1, 2]);
  });
});
