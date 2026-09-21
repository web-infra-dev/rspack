import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    // A chunk that gains css is resolved through the stale (pre-apply)
    // runtime's filename function, so the css filename must be derivable
    // from the chunk id alone — a shared hash-less template makes the
    // generated function dynamic instead of a fixed per-chunk map.
    cssFilename: '[name].css',
    cssChunkFilename: '[name].css',
  },
});
