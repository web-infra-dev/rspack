import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

class Plugin {
  apply(compiler: Compiler) {
    const { ContextModule } = compiler.rspack;

    compiler.hooks.afterEmit.tap('PLUGIN', (compilation) => {
      const contextModule = Array.from(compilation.modules).find(
        (module) => module instanceof ContextModule,
      )!;
      expect(contextModule.buildInfo).toBe(contextModule.buildInfo);
      expect(contextModule.buildInfo.fileDependencies).toBeInstanceOf(Set);
      expect(contextModule.buildInfo.contextDependencies).toBeInstanceOf(Set);
      expect(contextModule.factoryMeta).toBeTypeOf('object');
      expect(contextModule.layer).toBeUndefined();

      expect(contextModule.constructor.name).toBe('ContextModule');
      expect(contextModule.type).toBe('javascript/auto');
      expect(contextModule.useSourceMap).toBe(false);
      expect(contextModule.useSimpleSourceMap).toBe(false);

      expect(Object.hasOwn(contextModule, 'context')).toBe(true);
      expect(Object.hasOwn(contextModule, 'layer')).toBe(true);
      expect(Object.hasOwn(contextModule, 'factoryMeta')).toBe(true);
      expect(Object.hasOwn(contextModule, 'useSourceMap')).toBe(true);
      expect(Object.hasOwn(contextModule, 'useSimpleSourceMap')).toBe(true);
      expect(Object.hasOwn(contextModule, 'buildMeta')).toBe(true);
      expect(Object.hasOwn(contextModule, 'buildInfo')).toBe(true);
    });
  }
}

export default defineConfig({
  plugins: [new Plugin()],
});
