const shared = { value: 42 };
let building = false;
const options = {
  get transport() {
    if (building)
      throw new Error('Loader options were read on main during make');
    return 'native bytes';
  },
  shared,
  alias: shared,
  map: new Map([['shared', shared]]),
  set: new Set([shared]),
  buffer: Buffer.from('options'),
  typed: new Uint16Array([256, 1024]),
  date: new Date('2020-01-02T00:00:00Z'),
  regexp: /loader/gi,
  bigint: 12345678901234567890n,
  missing: undefined,
};
options.self = options;

export default {
  plugins: [
    {
      apply(compiler) {
        building = false;
        compiler.hooks.make.tap('SerializedLoaderOptions', () => {
          building = true;
        });
      },
    },
  ],
  module: {
    rules: [
      {
        test: /resource\.js$/,
        use: [
          './check-loader.js',
          {
            loader: './worker-loader.js',
            parallel: { maxWorkers: 2 },
            options,
          },
          'builtin:test-passthrough-loader',
          { loader: './worker-loader.js', parallel: true, options },
          './seed-loader.js',
        ],
      },
    ],
  },
};
