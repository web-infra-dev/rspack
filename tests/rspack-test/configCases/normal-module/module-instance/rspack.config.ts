import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import path from 'node:path';

class Plugin {
  type: string;
  layer: string | undefined;
  inline: boolean;

  constructor(type: string, layer: string | undefined, inline: boolean) {
    this.type = type;
    this.layer = layer;
    this.inline = inline;
  }

  apply(compiler: Compiler) {
    const { NormalModule } = compiler.rspack;

    compiler.hooks.afterEmit.tap('AutoExternalPlugin', (compilation) => {
      const normalModules = Array.from(compilation.modules).filter(
        (module) => module instanceof NormalModule,
      );

      expect(normalModules.length).toBe(1);

      const module = normalModules[0];

      expect(module instanceof NormalModule).toBe(true);
      expect(module.constructor.name).toBe('NormalModule');

      expect(module.resource).toContain('index.js');
      const request = `${path.join(import.meta.dirname, 'passthrough-loader.mjs')}!${path.join(import.meta.dirname, 'index.js')}`;
      expect(module.userRequest).toBe(this.inline ? request : module.resource);
      expect(module.request).toBe(request);
      expect(module.identifier()).toBe(
        this.layer
          ? `${this.type}|${module.request}|${this.layer}`
          : this.type === 'javascript/auto'
            ? module.request
            : `${this.type}|${module.request}`,
      );
      expect(module.rawRequest).toContain('index.js');
      expect(module.resourceResolveData?.fragment).toBe('');
      expect(module.resourceResolveData?.path).toBe(
        path.join(import.meta.dirname, 'index.js'),
      );
      expect(module.resourceResolveData?.query).toBe('');
      expect(module.resourceResolveData?.resource).toBe(
        path.join(import.meta.dirname, 'index.js'),
      );
      expect(module.loaders.length).toBe(1);
      expect(
        module.loaders.map(({ loader }) =>
          path.relative(compiler.context, loader),
        ),
      ).toEqual(['passthrough-loader.mjs']);
      expect(module.type).toBe(this.type);
      expect(module.layer).toBe(this.layer);

      expect(Object.hasOwn(module, 'type')).toBe(true);
      expect(Object.hasOwn(module, 'resource')).toBe(true);
      expect(Object.hasOwn(module, 'userRequest')).toBe(true);
      expect(Object.hasOwn(module, 'rawRequest')).toBe(true);
      expect(Object.hasOwn(module, 'resourceResolveData')).toBe(true);
      expect(Object.hasOwn(module, 'loaders')).toBe(true);
      expect(module.useSourceMap).toBe(true);
      expect(module.useSimpleSourceMap).toBe(false);
      expect(Object.hasOwn(module, 'matchResource')).toBe(true);
      expect(Object.hasOwn(module, 'layer')).toBe(true);
      expect(Object.hasOwn(module, 'factoryMeta')).toBe(true);
      expect(Object.hasOwn(module, 'buildMeta')).toBe(true);
      expect(Object.hasOwn(module, 'buildMeta')).toBe(true);
    });
  }
}

export default defineConfig(
  (
    [
      ['javascript/auto', undefined],
      ['javascript/esm', undefined],
      ['javascript/auto', 'test-layer'],
      ['javascript/auto', undefined, false],
    ] as const
  ).map(([type, layer, inline = true]) => ({
    devtool: 'source-map',
    entry: inline ? './passthrough-loader.mjs!./index.js' : './index.js',
    module: {
      rules: [
        {
          test: /index\.js$/,
          type,
          layer,
          use: inline ? [] : ['./passthrough-loader.mjs'],
        },
      ],
    },
    plugins: [new Plugin(type, layer, inline)],
  })),
);
