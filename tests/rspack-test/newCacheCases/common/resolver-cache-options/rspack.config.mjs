import path from 'node:path';

/** @type {import('@rspack/core').Configuration} */
export default {
  mode: 'development',
  experiments: {
    newCache: {
      resolver: true,
      module: false,
      codeGeneration: false,
      devtool: false,
      loader: false,
      minimize: false,
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'ResolverCacheOptionsTest',
          (compilation) => {
            compilation.hooks.finishModules.tapPromise(
              'ResolverCacheOptionsTest',
              async () => {
                const cases = [
                  [
                    'normal',
                    { alias: { target: './first.js' } },
                    'target',
                    'first.js',
                  ],
                  [
                    'normal',
                    { alias: { target: './second.js' } },
                    'target',
                    'second.js',
                  ],
                  [
                    'normal',
                    { alias: { target: './first.js' } },
                    'target?one#first',
                    'first.js?one#first',
                  ],
                  [
                    'normal',
                    { alias: { target: './first.js' } },
                    'target?two#second',
                    'first.js?two#second',
                  ],
                  [
                    'loader',
                    { alias: { target: './first.js' } },
                    'target',
                    'first.js',
                  ],
                  ['normal', {}, './directory', 'directory/index.js'],
                  ['context', {}, './directory', 'directory'],
                  ['normal', {}, './index.js', 'index.js'],
                  [
                    'normal',
                    {},
                    './index.js',
                    'directory/index.js',
                    'directory',
                  ],
                  ['normal', { alias: { target: false } }, 'target', false],
                ];
                for (const [
                  type,
                  options,
                  request,
                  expected,
                  directory = '',
                ] of cases) {
                  const resolver = compiler.resolverFactory.get(type, options);
                  const resolve = async () => {
                    const dependencies = {
                      fileDependencies: new Set(),
                      missingDependencies: new Set(),
                    };
                    const result = await new Promise((resolve, reject) => {
                      resolver.resolve(
                        {},
                        path.join(compiler.context, directory),
                        request,
                        dependencies,
                        (error, result) => {
                          if (error) reject(error);
                          else resolve(result);
                        },
                      );
                    });
                    return {
                      result,
                      files: [...dependencies.fileDependencies].sort(),
                      missing: [...dependencies.missingDependencies].sort(),
                    };
                  };
                  const first = await resolve();
                  expect(first.result).toBe(
                    expected === false
                      ? false
                      : path.join(compiler.context, expected),
                  );
                  expect(await resolve()).toEqual(first);
                }
              },
            );
          },
        );
      },
    },
  ],
};
