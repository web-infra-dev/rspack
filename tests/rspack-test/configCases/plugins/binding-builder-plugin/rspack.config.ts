import { defineConfig } from '@rspack/cli';
import { experiments } from '@rspack/core';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);

const binding = require(process.env.RSPACK_BINDING!);
binding.registerBindingBuilderTestingPlugin();

const BindingBuilderTestingPlugin = experiments.createNativePlugin(
  'BindingBuilderTestingPlugin',
  (options) => options,
);

export default defineConfig({
  plugins: [
    new BindingBuilderTestingPlugin({
      foo: 'bar',
    }),
  ],
});
