import assert from 'node:assert/strict';
import { rspack } from '@rspack/core';
import {
  closeCompiler,
  runCompiler,
} from '@rspack/test-tools/helper/lifecycle';

export default async function run() {
  const { ServerPlugin, ClientPlugin } = rspack.experiments.rsc.createPlugins();
  let startClientMake;
  const clientMaking = new Promise((resolve) => {
    startClientMake = resolve;
  });
  const compiler = rspack([
    {
      name: 'server',
      mode: 'none',
      target: 'node',
      entry: {},
      plugins: [
        new ServerPlugin(),
        {
          apply(compiler) {
            compiler.hooks.finishMake.tapPromise(
              { name: 'RejectFinishMake', stage: -10 },
              async () => {
                await clientMaking;
                throw new Error('server finishMake failed');
              },
            );
          },
        },
      ],
    },
    {
      name: 'client',
      mode: 'none',
      entry: {},
      plugins: [
        new ClientPlugin(),
        {
          apply(compiler) {
            compiler.hooks.make.tap('ClientMaking', startClientMake);
          },
        },
      ],
    },
  ]);

  try {
    await assert.rejects(runCompiler(compiler), /server finishMake failed/);
  } finally {
    await closeCompiler(compiler);
  }
}
