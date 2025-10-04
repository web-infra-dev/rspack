let globalId = 0;

const processAssets = jest.fn();
const afterProcessAssets = jest.fn();
const childCompiler = jest.fn();
const log = jest.fn();
const additionalAssets = jest.fn();
const optimizeModules = jest.fn();
const afterOptimizeModules = jest.fn();
const optimizeTree = jest.fn();
const optimizeChunkModules = jest.fn();
const finishModules = jest.fn();
const chunkHash = jest.fn();
const chunkAsset = jest.fn();
const processWarnings = jest.fn();
const succeedModule = jest.fn();
const stillValidModule = jest.fn();
const statsPreset = jest.fn();
const statsNormalize = jest.fn();
const statsFactory = jest.fn();
const statsPrinter = jest.fn();
const buildModule = jest.fn();
const executeModule = jest.fn();
const additionalTreeRuntimeRequirements = jest.fn();
const runtimeRequirementInTree = jest.fn();
const runtimeModule = jest.fn();
const seal = jest.fn();
const afterSeal = jest.fn();
const needAdditionalPass = jest.fn();
const addEntry = jest.fn();
const succeedEntry = jest.fn();
const failedEntry = jest.fn();

class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Plugin", compilation => {
			const localId = globalId += 1;

			compilation.hooks.processAssets.tap("Plugin", () => {
				processAssets();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.afterProcessAssets.tap("Plugin", () => {
				afterProcessAssets();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.childCompiler.tap("Plugin", () => {
				childCompiler();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.log.tap("Plugin", () => {
				log();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.additionalAssets.tap("Plugin", () => {
				additionalAssets();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.optimizeModules.tap("Plugin", () => {
				optimizeModules();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.afterOptimizeModules.tap("Plugin", () => {
				afterOptimizeModules();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.optimizeTree.tap("Plugin", () => {
				optimizeTree();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.optimizeChunkModules.tap("Plugin", () => {
				optimizeChunkModules();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.finishModules.tap("Plugin", () => {
				finishModules();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.chunkHash.tap("Plugin", () => {
				chunkHash();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.chunkAsset.tap("Plugin", () => {
				chunkAsset();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.processWarnings.tap("Plugin", () => {
				processWarnings();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.buildModule.tap("Plugin", () => {
				buildModule();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.executeModule.tap("Plugin", () => {
				executeModule();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.stillValidModule.tap("Plugin", () => {
				stillValidModule();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.statsPreset.tap("Plugin", () => {
				statsPreset();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.statsNormalize.tap("Plugin", () => {
				statsNormalize();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.statsFactory.tap("Plugin", () => {
				statsFactory();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.statsPrinter.tap("Plugin", () => {
				statsPrinter();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.runtimeRequirementInTree.tap("Plugin", () => {
				runtimeRequirementInTree();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.runtimeModule.tap("Plugin", () => {
				runtimeModule();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.needAdditionalPass.tap("Plugin", () => {
				needAdditionalPass();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.additionalTreeRuntimeRequirements.tap("Plugin", () => {
				additionalTreeRuntimeRequirements();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.seal.tap("Plugin", () => {
				seal();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.afterSeal.tap("Plugin", () => {
				afterSeal();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.addEntry.tap("Plugin", () => {
				addEntry();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.succeedEntry.tap("Plugin", () => {
				succeedEntry();
				expect(localId).toBe(globalId);
			});

			compilation.hooks.failedEntry.tap("Plugin", () => {
				failedEntry();
				expect(localId).toBe(globalId);
			});
		});
	}
}

/** @type {import('@rspack/core').TCompilerCaseConfig} */
module.exports = {
	description: "should be called every compilation",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()]
		};
	},
	async build(_, compiler) {
		await new Promise((resolve, reject) => {
			compiler.run(err => {
				if (err) {
					reject(err);
					return;
				}

				compiler.run(err => {
					if (err) {
						reject(err);
						return;
					}

					resolve();
				});
			});
		});
	},
	async check() {
		expect(processAssets.mock.calls.length).toBe(2);
		expect(afterProcessAssets.mock.calls.length).toBe(2);
		expect(childCompiler.mock.calls.length).toBe(2);
		expect(log.mock.calls.length).toBe(2);
		expect(additionalAssets.mock.calls.length).toBe(2);
		expect(optimizeModules.mock.calls.length).toBe(2);
		expect(afterOptimizeModules.mock.calls.length).toBe(2);
		expect(optimizeTree.mock.calls.length).toBe(2);
		expect(optimizeChunkModules.mock.calls.length).toBe(2);
		expect(finishModules.mock.calls.length).toBe(2);
		expect(chunkHash.mock.calls.length).toBe(2);
		expect(chunkAsset.mock.calls.length).toBe(2);
		expect(processWarnings.mock.calls.length).toBe(2);
		expect(buildModule.mock.calls.length).toBe(2);
		expect(executeModule.mock.calls.length).toBe(2);
		expect(stillValidModule.mock.calls.length).toBe(2);
		expect(statsPreset.mock.calls.length).toBe(2);
		expect(statsNormalize.mock.calls.length).toBe(2);
		expect(statsFactory.mock.calls.length).toBe(2);
		expect(statsPrinter.mock.calls.length).toBe(2);
		expect(runtimeRequirementInTree.mock.calls.length).toBe(2);
		expect(runtimeModule.mock.calls.length).toBe(2);
		expect(needAdditionalPass.mock.calls.length).toBe(2);
		expect(additionalTreeRuntimeRequirements.mock.calls.length).toBe(2);
		expect(seal.mock.calls.length).toBe(2);
		expect(afterSeal.mock.calls.length).toBe(2);
		expect(addEntry.mock.calls.length).toBe(0);
		expect(succeedEntry.mock.calls.length).toBe(2);
		expect(failedEntry.mock.calls.length).toBe(2);
	}
};
