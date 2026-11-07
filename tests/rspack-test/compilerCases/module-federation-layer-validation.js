const { container, sharing } = require('@rspack/core');

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
  {
    description: 'should reject empty federation layer names',
    build() {},
    check({ compiler }) {
      expect(() => new container.ContainerPlugin({
        enhanced: true,
        name: 'layers',
        exposes: { './module': { import: './module', layer: '' } },
      })).toThrow(/layer.*non-empty/);
      expect(() => new sharing.ProvideSharedPlugin({
        enhanced: true,
        provides: { shared: { layer: '' } },
      })).toThrow(/layer.*non-empty/);
      for (const field of ['layer', 'issuerLayer']) {
        expect(() => new sharing.ConsumeSharedPlugin({
          enhanced: true,
          consumes: { shared: { [field]: '' } },
        })).toThrow(new RegExp(`${field}.*non-empty`));
        expect(() => new sharing.SharePlugin({
          enhanced: true,
          shared: { shared: { [field]: '' } },
        }).apply(compiler)).toThrow(new RegExp(`${field}.*non-empty`));
      }
    },
  },
];
