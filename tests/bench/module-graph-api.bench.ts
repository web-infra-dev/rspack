import { type Compilation, type Module, rspack } from '@rspack/core';
import {
  beforeAll,
  bench as vitestBench,
  describe,
  type BenchmarkAPI,
} from 'vitest';
import rspackConfig from './fixtures/ts-react/rspack.config';

let theCompilation: Compilation;
let allDependencies: Module['dependencies'] = [];
let modulesWithConnections: Module[] = [];

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

beforeAll(() => {
  allDependencies = [];
  modulesWithConnections = [];

  return new Promise((resolve, reject) =>
    rspack(
      {
        ...rspackConfig,
        mode: 'production',
        plugins: [
          ...(rspackConfig.plugins ?? []),
          (compiler) => {
            compiler.hooks.compilation.tap(
              'MODULE_GRAPH_API_BENCH',
              (compilation) => {
                theCompilation = compilation;
              },
            );
          },
        ],
      },
      (err, stats) => {
        if (err) {
          reject(err);
          return;
        }
        if (stats?.hasErrors()) {
          reject(new Error(stats.toString({})));
          return;
        }

        for (const module of theCompilation.modules) {
          allDependencies.push(...module.dependencies);
        }

        modulesWithConnections = Array.from(theCompilation.modules).filter(
          (module) =>
            theCompilation.moduleGraph.getOutgoingConnections(module).length >
            0,
        );

        resolve(undefined);
      },
    ),
  );
});

describe('ModuleGraph API', () => {
  bench('ModuleGraph#getConnection', () => {
    for (const dependency of allDependencies) {
      theCompilation.moduleGraph.getConnection(dependency);
    }
  });

  bench('ModuleGraph#getModule', () => {
    for (const dependency of allDependencies) {
      theCompilation.moduleGraph.getModule(dependency);
    }
  });

  bench('ModuleGraph#getResolvedModule', () => {
    for (const dependency of allDependencies) {
      theCompilation.moduleGraph.getResolvedModule(dependency);
    }
  });

  bench('ModuleGraph#getOutgoingConnections', () => {
    for (const module of modulesWithConnections) {
      theCompilation.moduleGraph.getOutgoingConnections(module);
    }
  });

  bench('ModuleGraph#getOutgoingConnectionsInOrder', () => {
    for (const module of modulesWithConnections) {
      theCompilation.moduleGraph.getOutgoingConnectionsInOrder(module);
    }
  });

  bench('ModuleGraph#getIncomingConnections', () => {
    for (const module of modulesWithConnections) {
      theCompilation.moduleGraph.getIncomingConnections(module);
    }
  });
});
