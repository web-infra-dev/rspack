import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import path from 'node:path';

class CheckResolverPathsPlugin {
  apply(compiler: Compiler) {
    const first = compiler.resolverFactory.get('normal', {
      extensions: ['.js'],
    });
    const second = compiler.resolverFactory.get('normal', {
      extensions: ['.js', '.json'],
    });

    const forward = path.join(import.meta.dirname, 'forward.js');
    const forwardWithSlashes = forward.replaceAll('\\', '/');
    expect(first.resolveSync({}, import.meta.dirname, forwardWithSlashes)).toBe(
      forward,
    );
    expect(first.resolveSync({}, import.meta.dirname, forwardWithSlashes)).toBe(
      forward,
    );
    expect(second.resolveSync({}, import.meta.dirname, forward)).toBe(forward);

    const reverse = path.join(import.meta.dirname, 'reverse.js');
    const reverseWithSlashes = reverse.replaceAll('\\', '/');
    expect(second.resolveSync({}, import.meta.dirname, reverse)).toBe(reverse);
    expect(second.resolveSync({}, import.meta.dirname, reverse)).toBe(reverse);
    expect(first.resolveSync({}, import.meta.dirname, reverseWithSlashes)).toBe(
      reverse,
    );
  }
}

export default defineConfig({
  entry: './index.js',
  plugins: [new CheckResolverPathsPlugin()],
});
