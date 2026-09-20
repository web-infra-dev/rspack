import { Compiler } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    minimize: true,
    minimizer: [
      {
        /**
         * @param {Compiler} compiler the compiler
         */
        apply(compiler) {
          expect(compiler).toBeInstanceOf(Compiler);
        },
      },
      /**
       * @this {Compiler} the compiler
       * @param {Compiler} compiler the compiler
       */
      function (compiler) {
        expect(compiler).toBe(this);
        expect(compiler).toBeInstanceOf(Compiler);
      },
    ],
  },
};
