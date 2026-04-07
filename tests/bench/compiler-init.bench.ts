import {
  afterEach,
  bench as vitestBench,
  describe,
  type BenchmarkAPI,
} from 'vitest';
import binding from './helpers/rspack-binding';
import {
  builtinPlugins,
  noopRegisterJsTaps,
  noopThreadsafeNodeFS,
  rawCompilerPlatform,
  rawOptions,
} from './fixtures/compiler-init/binding-options';

// Mark benchmarks on JavaScript files with `js@` prefix
const bench = ((name, ...args) =>
  vitestBench(
    typeof name === 'function' ? name : `js@${name}`,
    ...args,
  )) as BenchmarkAPI;
bench.fn = vitestBench.fn;
bench.todo = vitestBench.todo;
bench.only = vitestBench.only;
bench.skip = vitestBench.skip;
bench.skipIf = vitestBench.skipIf;
bench.runIf = vitestBench.runIf;

describe('Compiler initialization', () => {
  const resolverFactory = new binding.JsResolverFactory(
    rawOptions.resolve.pnp ?? false,
    rawOptions.resolve,
    rawOptions.resolveLoader,
  );
  let compiler: binding.JsCompiler | undefined;

  afterEach(async () => {
    if (!compiler) {
      return;
    }
    await compiler.close();
    compiler = undefined;
  });

  bench('JsCompiler init via @rspack/binding', () => {
    compiler = new binding.JsCompiler(
      'bench|compiler-init|',
      rawOptions,
      builtinPlugins,
      noopRegisterJsTaps,
      noopThreadsafeNodeFS,
      undefined,
      undefined,
      resolverFactory,
      false,
      rawCompilerPlatform,
    );
  });
});
