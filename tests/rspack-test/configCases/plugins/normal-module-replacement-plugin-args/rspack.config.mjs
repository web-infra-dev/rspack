import { NormalModuleReplacementPlugin } from '@rspack/core';
export default /** @type {import("@rspack/core").Configuration} */ ({
  entry: {
    main: {
      import: './index.js',
      layer: 'test',
    },
  },
  plugins: [
    new NormalModuleReplacementPlugin(/request.v1(\.|$)/, (args) => {
      expect(args.request).toBe('./request.v1');
      expect(args.contextInfo.issuerLayer).toBe('test');
      expect(args.contextInfo.issuer.endsWith('index.js')).toBe(true);
      args.request = args.request.replace(/request\.v1(\.|$)/, 'request.v2$1');
    }),
  ],
});
