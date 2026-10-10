import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';
import binding, { type JsLoaderContext } from '@rspack/binding';

type Runner = (context: JsLoaderContext) => Promise<JsLoaderContext>;
const lazyFields = [
  'dependencies',
  'addedDependencies',
  'removedDependencies',
  'loaderItems',
  'sourceMap',
  'additionalData',
  '__internal__parseMeta',
] as const;

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /fixture\.js$/,
        use: [
          './consumer.mjs',
          'builtin:test-passthrough-loader',
          './producer.mjs',
        ],
      },
    ],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        const plugin = compiler.__internal__builtinPlugins.find(
          (plugin) =>
            plugin.name === binding.BuiltinPluginName.JsLoaderRspackPlugin,
        )!;
        const run = plugin.options as Runner;
        const instances = new Map<number, JsLoaderContext>();
        const invocations = new Map<number, number>();
        const reads = new WeakMap<JsLoaderContext, Map<string, number>>();
        let descriptors: Map<string, PropertyDescriptor> | undefined;
        let prototype: object;
        plugin.options = async (context: JsLoaderContext) => {
          assert.equal(
            Object.getPrototypeOf(context).constructor.name,
            'JsLoaderContext',
          );
          const id = context.id;
          if (instances.has(id)) assert.equal(context, instances.get(id));
          instances.set(id, context);
          invocations.set(id, (invocations.get(id) ?? 0) + 1);
          for (const field of [
            'id',
            'resource',
            'hot',
            '_module',
            'content',
            'loaderIndex',
          ]) {
            assert.ok(
              'value' in Object.getOwnPropertyDescriptor(context, field)!,
            );
          }
          for (const field of lazyFields)
            assert.equal(Object.hasOwn(context, field), false);
          const counts = new Map<string, number>();
          reads.set(context, counts);
          if (!descriptors) {
            prototype = Object.getPrototypeOf(context);
            descriptors = new Map(
              lazyFields.map((field) => [
                field,
                Object.getOwnPropertyDescriptor(prototype, field)!,
              ]),
            );
            for (const [field, descriptor] of descriptors) {
              Object.defineProperty(prototype, field, {
                ...descriptor,
                get(this: JsLoaderContext) {
                  const counts = reads.get(this);
                  if (counts) counts.set(field, (counts.get(field) ?? 0) + 1);
                  return descriptor.get!.call(this);
                },
              });
            }
          }
          const pitching =
            context.loaderState === binding.JsLoaderState.Pitching;
          const result = await run(context);
          assert.equal(result, context);
          for (const count of counts.values()) assert.equal(count, 1);
          if (pitching) {
            assert.equal(counts.has('sourceMap'), false);
            assert.equal(counts.has('additionalData'), false);
            assert.equal(counts.has('__internal__parseMeta'), false);
          }
          return result;
        };
        compiler.hooks.afterCompile.tap('LoaderContextClass', () => {
          assert.equal(instances.size, 1);
          assert.equal([...invocations.values()][0], 4);
          for (const context of instances.values()) {
            for (const field of lazyFields) {
              assert.equal(Object.hasOwn(context, field), false);
              assert.throws(
                () => context[field],
                /outside the active JavaScript loader invocation/,
              );
            }
          }
          for (const [field, descriptor] of descriptors!)
            Object.defineProperty(prototype, field, descriptor);
        });
      },
    }),
  ],
});
