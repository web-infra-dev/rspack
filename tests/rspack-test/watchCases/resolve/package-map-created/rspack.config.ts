import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  plugins: [
    definePlugin((compiler) => {
      // The case is compiled in a temporary directory, which is the compiler context.
      compiler.options.resolve.packageMap = path.join(
        compiler.context,
        'package-map.json',
      );
    }),
  ],
});
