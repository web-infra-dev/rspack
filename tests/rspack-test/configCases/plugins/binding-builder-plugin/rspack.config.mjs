import { experiments } from '@rspack/core';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
/** @type {import("@rspack/core").experiments} */

const binding = require(process.env.RSPACK_BINDING);
binding.registerBindingBuilderTestingPlugin();

const BindingBuilderTestingPlugin = experiments.createNativePlugin(
  'BindingBuilderTestingPlugin',
  (options) => options,
);

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new BindingBuilderTestingPlugin({
      foo: 'bar',
    }),
  ],
};
