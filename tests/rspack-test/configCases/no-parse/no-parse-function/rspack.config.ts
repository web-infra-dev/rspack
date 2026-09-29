import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    noParse: function (content) {
      return /not-parsed/.test(content);
    },
  },
});
