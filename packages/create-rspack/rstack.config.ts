import { define } from 'rstack';

define.lib({
  tools: {
    rspack: {
      experiments: {
        runtimeMode: 'rspack',
      },
    },
  },
  lib: [{ format: 'esm', syntax: ['es2023'] }],
});
