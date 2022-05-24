import { promises as fs } from 'fs';
import * as path from 'path';
import assert from 'assert';

import { expect } from '@rspack/test-toolkit';

import { Rspack, RspackPlugin } from '../src/node/rspack';

const fixtureRoot = path.resolve(__dirname, './fixtures');

describe('rspack:node-plugin', () => {
  it('should work with plugins', async () => {
    const fixture = path.join(fixtureRoot, 'plugin');

    const plugin: RspackPlugin = {
      async load(id) {
        if (id.endsWith('foo.js')) {
          return {
            content: `${await fs.readFile(id, 'utf-8')};console.log("fooo");`,
            loader: 'js',
          };
        }
      },
      async resolve(source, importer) {
        if (source === 'foo') {
          const nativeResult = this.resolve('./foo', {
            resolveDir: path.dirname(importer),
          });
          console.log('nativeResult', nativeResult);

          return {
            uri: nativeResult.path,
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
        sourceMap: 'none',
      },
    });

    await rspack.build();
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
