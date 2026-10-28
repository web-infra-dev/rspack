import assert from 'node:assert/strict';
import { createContext, runInContext } from 'node:vm';
import { defineConfig, definePlugin } from '@rspack/cli';
import { Compilation, container } from '@rspack/core';

export default ['var', 'global', 'window', 'self', 'module'].map((type) =>
  defineConfig({
    mode: 'development',
    devtool: false,
    target: 'web',
    entry:
      type === 'var'
        ? {
            main: {
              import: './index.js',
              library: { type: 'var', name: 'ordinary' },
            },
          }
        : './index.js',
    experiments: { outputModule: type === 'module' },
    output: { publicPath: '', globalObject: 'globalThis' },
    plugins: [
      new container.ContainerPlugin({
        name: 'provider',
        filename: 'remoteEntry.js',
        library: { type, name: 'probe' },
        exposes: { './message': './index.js' },
      }),
      ...(type === 'module'
        ? []
        : [
            new container.ContainerPlugin({
              name: 'nested_provider',
              filename: 'nestedRemoteEntry.js',
              library: { type, name: ['namespace', 'probe'] },
              exposes: { './message': './index.js' },
            }),
          ]),
      definePlugin({
        apply(compiler) {
          compiler.hooks.compilation.tap(
            'AssertContainerGlobalCollision',
            (compilation) => {
              compilation.hooks.processAssets.tap(
                {
                  name: 'AssertContainerGlobalCollision',
                  stage: Compilation.PROCESS_ASSETS_STAGE_REPORT,
                },
                () => {
                  const source = compilation
                    .getAsset('remoteEntry.js')!
                    .source.source()
                    .toString();
                  if (type === 'module') {
                    assert(!source.includes('Cannot register container'));
                    return;
                  }
                  const makeContext = () => {
                    const context = createContext({});
                    runInContext('window = self = this;', context);
                    return context;
                  };
                  const fresh = makeContext();
                  runInContext(source, fresh);
                  assert.equal(typeof fresh.probe.get, 'function');
                  assert.equal(typeof fresh.probe.init, 'function');

                  const preloaded = { get() {}, init() {} };
                  const loaded = makeContext();
                  loaded.probe = preloaded;
                  runInContext(source, loaded);
                  assert.equal(loaded.probe, preloaded);

                  for (const value of [
                    0,
                    false,
                    '',
                    null,
                    {},
                    () => {},
                    Number.NaN,
                  ]) {
                    const context = makeContext();
                    context.probe = value;
                    assert.throws(
                      () => runInContext(source, context),
                      /global name is already in use/,
                    );
                    assert(Object.is(context.probe, value));
                  }

                  for (const value of [undefined, { get() {}, init() {} }]) {
                    const context = makeContext();
                    Object.defineProperty(context, 'probe', {
                      value,
                      writable: false,
                      configurable: false,
                    });
                    if (value) runInContext(source, context);
                    else
                      assert.throws(
                        () => runInContext(source, context),
                        /global name is already in use/,
                      );
                    assert.equal(context.probe, value);
                  }

                  const getter = makeContext();
                  let reads = 0;
                  Object.defineProperty(getter, 'probe', {
                    get() {
                      reads++;
                      return {};
                    },
                    configurable: true,
                  });
                  assert.throws(
                    () => runInContext(source, getter),
                    /global name is already in use/,
                  );
                  assert.equal(reads, 0);

                  const inherited = makeContext();
                  runInContext(
                    'Object.getPrototypeOf(this).probe = 42;',
                    inherited,
                  );
                  assert.throws(
                    () => runInContext(source, inherited),
                    /global name is already in use/,
                  );
                  assert.equal(
                    runInContext(
                      'Object.getPrototypeOf(this).probe',
                      inherited,
                    ),
                    42,
                  );

                  const nestedSource = compilation
                    .getAsset('nestedRemoteEntry.js')!
                    .source.source()
                    .toString();
                  const nested = makeContext();
                  runInContext(nestedSource, nested);
                  assert.equal(typeof nested.namespace.probe.get, 'function');
                  assert.equal(typeof nested.namespace.probe.init, 'function');
                  const occupied = makeContext();
                  occupied.namespace = { probe: 0, keep: 42 };
                  const original = occupied.namespace;
                  assert.throws(
                    () => runInContext(nestedSource, occupied),
                    /global name is already in use/,
                  );
                  assert.equal(occupied.namespace, original);
                  assert.equal(occupied.namespace.probe, 0);
                  assert.equal(occupied.namespace.keep, 42);

                  const partial = makeContext();
                  partial.namespace = { keep: 42 };
                  const namespace = partial.namespace;
                  runInContext(nestedSource, partial);
                  assert.equal(partial.namespace, namespace);
                  assert.equal(partial.namespace.keep, 42);
                  assert.equal(typeof partial.namespace.probe.get, 'function');

                  if (type === 'var') {
                    const lexical = makeContext();
                    runInContext('const probe = 42;', lexical);
                    assert.throws(
                      () => runInContext(source, lexical),
                      /already been declared/,
                    );
                    assert.equal(runInContext('probe', lexical), 42);

                    const ordinarySource = compilation
                      .getAsset('main.js')!
                      .source.source()
                      .toString();
                    const ordinary = makeContext();
                    ordinary.ordinary = 0;
                    runInContext(ordinarySource, ordinary);
                    assert.equal(ordinary.ordinary.message, 'federated-ok');
                  }
                },
              );
            },
          );
        },
      }),
    ],
  }),
);
