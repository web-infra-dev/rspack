import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    environment: {
      bigIntLiteral: true,
    },
  },
  plugins: [
    new DefinePlugin({
      BIGINT: BigInt('9007199254740993'),
      ZERO_BIGINT: BigInt(0),
    }),
  ],
};
