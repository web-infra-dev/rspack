import { MessageChannel } from 'node:worker_threads';

const { port1, port2 } = new MessageChannel();

export default [{ nested: { onDone() {} } }, { nested: { port: port1 } }].map(
  (options) => ({
    module: {
      rules: [
        {
          test: /lib\.js$/,
          use: [
            {
              loader: './loader.mjs',
              parallel: true,
              options,
            },
          ],
        },
      ],
    },
    plugins:
      'port' in options.nested
        ? [
            {
              apply(compiler) {
                compiler.hooks.done.tap('ClosePorts', () => {
                  port1.close();
                  port2.close();
                });
              },
            },
          ]
        : [],
  }),
);
