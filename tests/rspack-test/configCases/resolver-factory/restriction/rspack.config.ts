import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import path from 'node:path';

class Plugin {
  apply(compiler: Compiler) {
    const normalResolver = compiler.resolverFactory.get('normal');
    const newResolver = normalResolver.withOptions({
      extensions: ['.css'],
      mainFields: ['main'],
      restrictions: [/\.css$/],
    });
    const request = newResolver.resolveSync({}, import.meta.dirname, 'style');
    expect(request).toBe(
      path.join(import.meta.dirname, '/node_modules/style/index.css'),
    );
  }
}

export default defineConfig({
  entry: './index.js',
  plugins: [new Plugin()],
});
