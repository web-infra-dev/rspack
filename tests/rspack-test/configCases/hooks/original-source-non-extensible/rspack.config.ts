import { createHash } from 'node:crypto';
import { defineConfig } from '@rspack/cli';
import { type Compiler, NormalModule, sources } from '@rspack/core';
import type { Source } from 'webpack-sources';

const restrictions = [Object.seal, Object.freeze, Object.preventExtensions];

function outcome(operation: () => unknown) {
  try {
    return { value: operation() };
  } catch (error) {
    if (!(error instanceof TypeError)) throw error;
    return { error: error.name };
  }
}

function hash(source: Source) {
  const hash = createHash('sha256');
  source.updateHash(hash);
  return hash.digest('hex');
}

export default ([false, 'source-map'] as const).flatMap((devtool) =>
  restrictions.map((restrictModule) =>
    defineConfig({
      devtool,
      plugins: [
        (compiler: Compiler) => {
          compiler.hooks.done.tap('NonExtensibleSources', ({ compilation }) => {
            const module = [...compilation.modules].find(
              (module) =>
                module instanceof NormalModule &&
                module.resource.endsWith('value.js'),
            )!;
            expect(module).toBeDefined();
            restrictModule(module);
            const moduleKeys = Reflect.ownKeys(module);
            const original = module.originalSource()!;
            // Create every instance before consuming the shared map so each retains its getter.
            const pending = [true, false].flatMap((cacheBuffers) =>
              restrictions.map((restrictSource) => ({
                cacheBuffers,
                restrictSource,
                source: module.originalSource()!,
              })),
            );
            const value = original.source();
            // Materialize the shared map through a restricted instance for the first time.
            restrictModule(original);
            const json =
              original instanceof sources.SourceMapSource
                ? original._sourceMapAsString
                : undefined;
            const caching = sources.util.stringBufferUtils;
            const wasCaching = caching.isDualStringBufferCachingEnabled();
            try {
              for (const { cacheBuffers, restrictSource, source } of pending) {
                if (cacheBuffers) caching.enableDualStringBufferCaching();
                else caching.disableDualStringBufferCaching();
                const reference = json
                  ? new sources.SourceMapSource(
                      value,
                      'inmemory://from rust',
                      json,
                    )
                  : new sources.RawSource(value);
                if (json) {
                  expect(
                    Object.getOwnPropertyDescriptor(
                      source,
                      '_sourceMapAsString',
                    )!.get,
                  ).toBeTypeOf('function');
                }
                restrictSource(source);
                restrictSource(reference);
                const operations: ((source: Source) => unknown)[] = [
                  (source) => source.source(),
                  (source) =>
                    source instanceof sources.SourceMapSource
                      ? source._sourceMapAsString
                      : undefined,
                  (source) => source.map(),
                  (source) => source.sourceAndMap(),
                  hash,
                  (source) =>
                    source instanceof sources.SourceMapSource
                      ? source.getArgsAsBuffers()
                      : source.buffer(),
                  (source) => source.clearCache(),
                  (source) => source.map(),
                  hash,
                  (source) => source.clearCache({ parsedMap: true }),
                  (source) => source.sourceAndMap(),
                  hash,
                ];
                for (const operation of operations) {
                  expect(outcome(() => operation(source))).toEqual(
                    outcome(() => operation(reference)),
                  );
                }
              }
            } finally {
              if (wasCaching) caching.enableDualStringBufferCaching();
              else caching.disableDualStringBufferCaching();
            }
            expect(Reflect.ownKeys(module)).toEqual(moduleKeys);
            expect(module.originalSource()!.sourceAndMap()).toEqual({
              source: value,
              map: json ? JSON.parse(json) : null,
            });
          });
        },
      ],
    }),
  ),
);
