import assert from 'node:assert/strict';

function stackThroughPluginCall(error) {
  assert(error instanceof Error);
  const lines = error.stack.split('\n');
  const pluginFrame = lines.findIndex((line) =>
    /^\s+at .*module-graph-stack\.mjs:\d+:\d+\)$/.test(line),
  );
  assert(pluginFrame > 0, error.stack);
  return lines
    .slice(0, pluginFrame + 1)
    .map((line) =>
      line
        // Node 22 displays the receiver of this static method as Function.
        .replace('at Function.__from_binding (', 'at EntryData.__from_binding (')
        .replace(/\(.*[/\\]([^/\\]+):\d+:\d+\)$/, '($1:<line>:<column>)'),
    )
    .join('\n');
}

function thrownStack(fn) {
  try {
    fn();
  } catch (error) {
    return stackThroughPluginCall(error);
  }
  assert.fail('Expected the graph API to throw');
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  {
    description:
      'should throw catchable graph errors with the plugin stack during make',
    async compiler(_, compiler) {
      let checkedBuildModule = false;
      compiler.hooks.thisCompilation.tap('ModuleGraphStack', (compilation) => {
        compilation.hooks.buildModule.tap('ModuleGraphStack', (module) => {
          const { moduleGraph } = compilation;
          expect(
            thrownStack(function readIssuerDuringMake() {
              moduleGraph.getIssuer(module);
            }),
          ).toMatchInlineSnapshot(`
            Error: ModuleGraph is not available during module graph building phase
                at ModuleGraph.getIssuer (index.js:<line>:<column>)
                at readIssuerDuringMake (module-graph-stack.mjs:<line>:<column>)
          `);
          expect(
            thrownStack(function readEntryDuringMake() {
              compilation.entries.get('main');
            }),
          ).toMatchInlineSnapshot(`
            Error: ModuleGraph is not available during module graph building phase
                at new EntryData (index.js:<line>:<column>)
                at EntryData.__from_binding (index.js:<line>:<column>)
                at Entries.get (index.js:<line>:<column>)
                at readEntryDuringMake (module-graph-stack.mjs:<line>:<column>)
          `);
          checkedBuildModule = true;
        });
      });
      compiler.hooks.finishMake.tap('ModuleGraphStack', (compilation) => {
        assert(checkedBuildModule);
        const { dependencies, includeDependencies } =
          compilation.entries.get('main');
        assert(compilation.moduleGraph.getModule(dependencies[0]));
        assert.deepEqual(includeDependencies, []);
      });
    },
  },
  {
    description:
      'should preserve the original make error stack in the compiler callback',
    async build(_, compiler) {
      let originalStack;
      compiler.hooks.thisCompilation.tap('ModuleGraphStack', (compilation) => {
        compilation.hooks.buildModule.tap(
          'ModuleGraphStack',
          function readIssuerDuringMake(module) {
            try {
              compilation.moduleGraph.getIssuer(module);
            } catch (error) {
              originalStack = error.stack;
              throw error;
            }
          },
        );
      });
      const error = await new Promise((resolve) => compiler.run(resolve));
      expect(stackThroughPluginCall(error)).toMatchInlineSnapshot(`
        Error: ModuleGraph is not available during module graph building phase
            at ModuleGraph.getIssuer (index.js:<line>:<column>)
            at Object.readIssuerDuringMake [as fn] (module-graph-stack.mjs:<line>:<column>)
      `);
      expect(error.message.split('\n', 1)[0].trim()).toMatchInlineSnapshot(
        `× Error: ModuleGraph is not available during module graph building phase`,
      );
      assert.equal(error.stack, originalStack);
    },
  },
  {
    description:
      'should report unavailable export info and async state during finishModules',
    async compiler(_, compiler) {
      let entryModule;
      let exportsInfo;
      compiler.hooks.finishMake.tap('ModuleGraphStack', (compilation) => {
        const { dependencies } = compilation.entries.get('main');
        entryModule = compilation.moduleGraph.getModule(dependencies[0]);
        exportsInfo = compilation.moduleGraph.getExportsInfo(entryModule);
      });
      compiler.hooks.thisCompilation.tap('ModuleGraphStack', (compilation) => {
        compilation.hooks.finishModules.tap('ModuleGraphStack', () => {
          expect(
            thrownStack(function readAsyncStateDuringFinishModules() {
              compilation.moduleGraph.isAsync(entryModule);
            }),
          ).toMatchInlineSnapshot(`
            Error: Module async state is not available while a compilation pass is updating it
                at ModuleGraph.isAsync (index.js:<line>:<column>)
                at readAsyncStateDuringFinishModules (module-graph-stack.mjs:<line>:<column>)
          `);
          expect(
            thrownStack(function readSavedExportsInfoDuringFinishModules() {
              exportsInfo.isUsed();
            }),
          ).toMatchInlineSnapshot(`
            Error: Exports info is not available while a compilation pass is updating it
                at ExportsInfo.isUsed (index.js:<line>:<column>)
                at readSavedExportsInfoDuringFinishModules (module-graph-stack.mjs:<line>:<column>)
          `);
          expect(
            thrownStack(function readProvidedExportsDuringFinishModules() {
              compilation.moduleGraph.getProvidedExports(entryModule);
            }),
          ).toMatchInlineSnapshot(`
            Error: Exports info is not available while a compilation pass is updating it
                at ModuleGraph.getProvidedExports (index.js:<line>:<column>)
                at readProvidedExportsDuringFinishModules (module-graph-stack.mjs:<line>:<column>)
          `);
        });
        compilation.hooks.afterOptimizeModules.tap('ModuleGraphStack', () => {
          assert.doesNotThrow(() =>
            compilation.moduleGraph.getProvidedExports(entryModule),
          );
          assert.doesNotThrow(() => exportsInfo.isUsed());
          assert.equal(compilation.moduleGraph.isAsync(entryModule), false);
        });
      });
    },
  },
];
