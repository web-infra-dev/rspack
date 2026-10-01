import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import { deepEqual } from 'node:assert';

class InterceptPlugin {
  apply(compiler: Compiler) {
    const content: string[] = [];
    compiler.hooks.beforeCompile.intercept({
      call() {
        content.push('compiler.hooks.beforeCompile.intercept.call');
      },
    });
    compiler.hooks.compile.intercept({
      call() {
        content.push('compiler.hooks.compile.intercept.call');
      },
    });
    compiler.hooks.finishMake.intercept({
      call() {
        content.push('compiler.hooks.finishMake.intercept.call');
      },
    });
    compiler.hooks.afterCompile.intercept({
      call() {
        content.push('compiler.hooks.afterCompile.intercept.call');
      },
    });
    compiler.hooks.done.tap(InterceptPlugin.name, () => {
      deepEqual(content, [
        'compiler.hooks.beforeCompile.intercept.call',
        'compiler.hooks.compile.intercept.call',
        'compiler.hooks.finishMake.intercept.call',
        'compiler.hooks.afterCompile.intercept.call',
      ]);
    });
  }
}

const config = defineConfig({
  plugins: [new InterceptPlugin()],
});

export default config;
