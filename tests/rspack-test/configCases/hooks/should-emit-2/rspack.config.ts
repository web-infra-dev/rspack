import assert from 'node:assert';
import fs from 'node:fs';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

const customBundleFile = `
it("should emit this file", () => {
	expect(3).toBe(3);
});
`;

class Plugin {
  apply(compiler: Compiler) {
    let count = 0;
    compiler.hooks.shouldEmit.tap('should-emit-should-works', (compilation) => {
      assert(typeof compilation !== 'undefined');
      assert(typeof compilation.hooks !== 'undefined');

      count += 1;
      const filePath = path.resolve(
        import.meta.dirname,
        compiler.options.output.path!,
        './bundle0.js',
      );
      if (!fs.existsSync(path.dirname(filePath))) {
        fs.mkdirSync(path.dirname(filePath), { recursive: true });
      }
      fs.writeFileSync(filePath, customBundleFile);
      return false;
    });

    compiler.hooks.done.tap('check', () => {
      assert(count === 1);
    });
  }
}

export default defineConfig({
  plugins: [new Plugin()],
  node: false,
});
