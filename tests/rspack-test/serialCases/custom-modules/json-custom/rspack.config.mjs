import toml from 'toml';

/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    mode: 'development',
    module: {
      rules: [
        {
          test: /\.toml$/,
          type: 'json',
          parser: {
            parse(input) {
              expect(arguments.length).toBe(1);
              return toml.parse(input);
            },
          },
        },
      ],
    },
  },
];
