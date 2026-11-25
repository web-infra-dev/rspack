import assert from 'node:assert/strict';

const moduleGraphUnavailable =
  'ModuleGraph is not available during module graph building phase';
const exportsInfoUnavailable =
  'Exports info is not available while a compilation pass is updating it';

function assertPluginStack(error, method, caller) {
  assert(error instanceof Error);
  const lines = error.stack.split('\n');
  const apiFrame = lines.findIndex((line) =>
    line.startsWith(`    at ${method} (`),
  );
  const pluginFrame = lines.findIndex(
    (line) =>
      line.includes(caller) &&
      /^\s+at .*[/\\]module-graph-stack\.mjs:\d+:\d+\)$/.test(line),
  );
  assert(apiFrame > 0 && pluginFrame > apiFrame, error.stack);
}

function assertGraphError(method, message, fn) {
  assert.throws(fn, (error) => {
    assertPluginStack(error, method, fn.name);
    assert.equal(error.message, message);
    return true;
  });
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
          assertGraphError(
            'ModuleGraph.getIssuer',
            moduleGraphUnavailable,
            function readIssuerDuringMake() {
              moduleGraph.getIssuer(module);
            },
          );
          assertGraphError(
            'Entries.get',
            moduleGraphUnavailable,
            function readEntryDuringMake() {
              compilation.entries.get('main');
            },
          );
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
      assertPluginStack(error, 'ModuleGraph.getIssuer', 'readIssuerDuringMake');
      assert.equal(
        error.message.split('\n', 1)[0].trim(),
        `× Error: ${moduleGraphUnavailable}`,
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
          assertGraphError(
            'ModuleGraph.isAsync',
            'Module async state is not available while a compilation pass is updating it',
            function readAsyncStateDuringFinishModules() {
              compilation.moduleGraph.isAsync(entryModule);
            },
          );
          assertGraphError(
            'ExportsInfo.isUsed',
            exportsInfoUnavailable,
            function readSavedExportsInfoDuringFinishModules() {
              exportsInfo.isUsed();
            },
          );
          assertGraphError(
            'ModuleGraph.getProvidedExports',
            exportsInfoUnavailable,
            function readProvidedExportsDuringFinishModules() {
              compilation.moduleGraph.getProvidedExports(entryModule);
            },
          );
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
