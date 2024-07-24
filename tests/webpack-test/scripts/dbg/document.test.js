/**
 * debug script for dist of ConfigTestCases
 * 
 * @example
 * after we have dist in /js/ConfigTestCases/${CATEGORY}/${TEST}
 * CATEGORY=entry TEST=no-chunking node --experimental-vm-modules --max-old-space-size=4096  --trace-deprecation ../node_modules/jest-cli/bin/jest  --logHeapUsage --runInBand --bail --forceExit -c ./scripts/dbg/jest.config.js ./scripts/dbg/document.test.js
 */

const path = require("path");
const fs = require("graceful-fs");
const vm = require("vm");
const { URL, pathToFileURL, fileURLToPath } = require("url");
const FakeDocument = require("../../helpers/FakeDocument");
const CurrentScript = require("../../helpers/CurrentScript");
const createLazyTestEnv = require("../../helpers/createLazyTestEnv");

const prepareOptions = require("../../helpers/prepareOptions");
const asModule = require("../../helpers/asModule");
const { parseResource } = require("../../lib/util/identifier");

const {
  it: _it,
  beforeEach: _beforeEach,
  afterEach: _afterEach,
} = createLazyTestEnv(10000);

const testRoot = path.resolve(__dirname, '../../')
const configName = "ConfigTestCases"
const categoryName = process.env['CATEGORY'];
const testName = process.env['TEST'];

const casesPath = path.join(testRoot, "configCases");
const outBaseDir = path.join(testRoot, "js");
const testSubPath = path.join(configName, categoryName, testName);
const testDirectory = path.join(casesPath, categoryName, testName);
const outputDirectory = path.join(outBaseDir, testSubPath);

let options, optionsArr, testConfig;
options = prepareOptions(
  require(path.join(testDirectory, "webpack.config.js")),
  { testPath: outputDirectory }
);
optionsArr = [].concat(options);
optionsArr.forEach((options, idx) => {
  if (!options.context) options.context = testDirectory;
  if (!options.mode) options.mode = "production";
  if (!options.optimization) options.optimization = {};
  if (options.optimization.minimize === undefined)
    options.optimization.minimize = false;
  // if (options.optimization.minimizer === undefined) {
  // 	options.optimization.minimizer = [
  // 		new (require("terser-webpack-plugin"))({
  // 			parallel: false
  // 		})
  // 	];
  // }
  if (!options.entry) options.entry = "./index.js";
  if (!options.target) options.target = "async-node";
  if (!options.output) options.output = {};
  if (!options.devtool) options.devtool = false;
  if (options.cache === undefined) options.cache = false;
  if (!options.output.path) options.output.path = outputDirectory;
  // if (typeof options.output.pathinfo === "undefined")
  // 	options.output.pathinfo = true;
  if (!options.output.filename)
    options.output.filename =
      "bundle" +
      idx +
      (options.experiments && options.experiments.outputModule
        ? ".mjs"
        : ".js");
  // if (config.cache) {
  //   options.cache = {
  //     cacheDirectory,
  //     name: `config-${idx}`,
  //     ...config.cache
  //   };
  //   options.infrastructureLogging = {
  //     debug: true,
  //     console: createLogger(infraStructureLog)
  //   };
  // }
  if (!options.snapshot) options.snapshot = {};
  // if (!options.snapshot.managedPaths) {
  // 	options.snapshot.managedPaths = [
  // 		path.resolve(__dirname, "../node_modules")
  // 	];
  // }
});
testConfig = {
  findBundle: function(i, options) {
    const ext = path.extname(
      parseResource(options.output.filename).path
    );
    if (
      fs.existsSync(
        path.join(options.output.path, "bundle" + i + ext)
      )
    ) {
      return "./bundle" + i + ext;
    }
  },
  timeout: 30000
};
try {
  // try to load a test file
  testConfig = Object.assign(
    testConfig,
    require(path.join(testDirectory, "test.config.js"))
  );
} catch (e) {
  // ignored
}

let filesCount = 0;

if (testConfig.beforeExecute) testConfig.beforeExecute();
const results = [];
for (let i = 0; i < optionsArr.length; i++) {
  const options = optionsArr[i];
  const bundlePath = testConfig.findBundle(i, optionsArr[i]);
  if (bundlePath) {
    filesCount++;
    const document = new FakeDocument(outputDirectory);
    const globalContext = {
      console: console,
      expect: expect,
      setTimeout: setTimeout,
      clearTimeout: clearTimeout,
      document,
      getComputedStyle:
        document.getComputedStyle.bind(document),
      location: {
        href: "https://test.cases/path/index.html",
        origin: "https://test.cases",
        toString() {
          return "https://test.cases/path/index.html";
        }
      }
    };

    const requireCache = Object.create(null);
    const esmCache = new Map();
    const esmIdentifier = `dbg-document-${i}`;
    const baseModuleScope = {
      console: console,
      it: _it,
      beforeEach: _beforeEach,
      afterEach: _afterEach,
      expect,
      jest,
      // __STATS__: jsonStats,
      nsObj: m => {
        Object.defineProperty(m, Symbol.toStringTag, {
          value: "Module"
        });
        return m;
      }
    };

    let runInNewContext = false;
    if (
      options.target === "web" ||
      options.target === "webworker"
    ) {
      baseModuleScope.window = globalContext;
      baseModuleScope.self = globalContext;
      baseModuleScope.URL = URL;
      baseModuleScope.Worker =
        require("../../helpers/createFakeWorker")({
          outputDirectory
        });
      runInNewContext = true;
    }
    if (testConfig.moduleScope) {
      testConfig.moduleScope(baseModuleScope);
    }
    const esmContext = vm.createContext(baseModuleScope, {
      name: "context for esm"
    });

    // eslint-disable-next-line no-loop-func
    const _require = (
      currentDirectory,
      options,
      module,
      esmMode,
      parentModule
    ) => {
      if (testConfig === undefined) {
        throw new Error(
          `_require(${module}) called after all tests from ${categoryName} ${testName} have completed`
        );
      }
      if (Array.isArray(module) || /^\.\.?\//.test(module)) {
        let content;
        let p;
        let subPath = "";
        if (Array.isArray(module)) {
          p = path.join(currentDirectory, ".array-require.js");
          content = `module.exports = (${module
            .map(arg => {
              return `require(${JSON.stringify(`./${arg}`)})`;
            })
            .join(", ")});`;
        } else {
          p = path.join(currentDirectory, module);
          content = fs.readFileSync(p, "utf-8");
          const lastSlash = module.lastIndexOf("/");
          let firstSlash = module.indexOf("/");

          if (lastSlash !== -1 && firstSlash !== lastSlash) {
            if (firstSlash !== -1) {
              let next = module.indexOf("/", firstSlash + 1);
              let dir = module.slice(firstSlash + 1, next);

              while (dir === ".") {
                firstSlash = next;
                next = module.indexOf("/", firstSlash + 1);
                dir = module.slice(firstSlash + 1, next);
              }
            }

            subPath = module.slice(
              firstSlash + 1,
              lastSlash + 1
            );
          }
        }
        const isModule =
          p.endsWith(".mjs") &&
          options.experiments &&
          options.experiments.outputModule;

        if (isModule) {
          if (!vm.SourceTextModule)
            throw new Error(
              "Running this test requires '--experimental-vm-modules'.\nRun with 'node --experimental-vm-modules node_modules/jest-cli/bin/jest'."
            );
          let esm = esmCache.get(p);
          if (!esm) {
            esm = new vm.SourceTextModule(content, {
              identifier: esmIdentifier + "-" + p,
              url: pathToFileURL(p).href + "?" + esmIdentifier,
              context: esmContext,
              initializeImportMeta: (meta, module) => {
                meta.url = pathToFileURL(p).href;
              },
              importModuleDynamically: async (
                specifier,
                module
              ) => {
                const result = await _require(
                  path.dirname(p),
                  options,
                  specifier,
                  "evaluated",
                  module
                );
                return await asModule(result, module.context);
              }
            });
            esmCache.set(p, esm);
          }
          if (esmMode === "unlinked") return esm;
          return (async () => {
            await esm.link(
              async (specifier, referencingModule) => {
                return await asModule(
                  await _require(
                    path.dirname(
                      referencingModule.identifier
                        ? referencingModule.identifier.slice(
                          esmIdentifier.length + 1
                        )
                        : fileURLToPath(referencingModule.url)
                    ),
                    options,
                    specifier,
                    "unlinked",
                    referencingModule
                  ),
                  referencingModule.context,
                  true
                );
              }
            );
            // node.js 10 needs instantiate
            if (esm.instantiate) esm.instantiate();
            await esm.evaluate();
            if (esmMode === "evaluated") return esm;
            const ns = esm.namespace;
            return ns.default && ns.default instanceof Promise
              ? ns.default
              : ns;
          })();
        } else {
          if (p in requireCache) {
            return requireCache[p].exports;
          }
          const m = {
            exports: {}
          };
          requireCache[p] = m;
          const moduleScope = {
            ...baseModuleScope,
            require: _require.bind(
              null,
              path.dirname(p),
              options
            ),
            importScripts: url => {
              expect(url).toMatch(
                /^https:\/\/test\.cases\/path\//
              );
              _require(
                outputDirectory,
                options,
                `.${url.slice(
                  "https://test.cases/path".length
                )}`
              );
            },
            module: m,
            exports: m.exports,
            __dirname: path.dirname(p),
            __filename: p,
            _globalAssign: { expect }
          };
          if (testConfig.moduleScope) {
            testConfig.moduleScope(moduleScope);
          }
          if (!runInNewContext)
            content = `Object.assign(global, _globalAssign); ${content}`;
          const args = Object.keys(moduleScope);
          const argValues = args.map(arg => moduleScope[arg]);
          const code = `(function(${args.join(
            ", "
          )}) {${content}\n})`;

          let oldCurrentScript = document.currentScript;
          document.currentScript = new CurrentScript(subPath);
          const fn = runInNewContext
            ? vm.runInNewContext(code, globalContext, p)
            : vm.runInThisContext(code, p);
          fn.call(
            testConfig.nonEsmThis
              ? testConfig.nonEsmThis(module)
              : m.exports,
            ...argValues
          );
          document.currentScript = oldCurrentScript;
          return m.exports;
        }
      } else if (
        testConfig.modules &&
        module in testConfig.modules
      ) {
        return testConfig.modules[module];
      } else {
        return require(module.startsWith("node:")
          ? module.slice(5)
          : module);
      }
    };

    if (Array.isArray(bundlePath)) {
      for (const bundlePathItem of bundlePath) {
        results.push(
          _require(
            outputDirectory,
            options,
            "./" + bundlePathItem
          )
        );
      }
    } else {
      results.push(
        _require(outputDirectory, options, bundlePath)
      );
    }
  }
}
