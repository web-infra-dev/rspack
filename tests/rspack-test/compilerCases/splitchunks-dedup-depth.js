/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [-1, 1.5, NaN, Infinity, 0x100000000, "2", null].map(
  (dedupDepth) => ({
    description: `rejects splitChunks.dedupDepth ${String(dedupDepth)}`,
    options(context) {
      return {
        context: context.getSource(),
        entry: "./a",
        optimization: { splitChunks: { dedupDepth } },
      };
    },
    async build(_, compiler) {
      await new Promise((resolve) => compiler.run(() => resolve()));
    },
    async check({ context, name }) {
      const errors = context.getError(name);
      expect(errors).toHaveLength(1);
      expect(errors[0].toString()).toContain(
        '"optimization.splitChunks.dedupDepth" must be an integer between 0 and 4294967295',
      );
      context.clearError(name);
    },
  }),
);
