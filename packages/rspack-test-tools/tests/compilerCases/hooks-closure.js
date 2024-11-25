let globalId = 0;

const buildModule = jest.fn();
const succeedModule = jest.fn();
const finishModules = jest.fn();
const optimizeModules = jest.fn();
const afterOptimizeModules = jest.fn();
const optimizeTree = jest.fn();
const optimizeChunkModules = jest.fn();
const additionalTreeRuntimeRequirements = jest.fn();
const runtimeModule = jest.fn();
const chunkHash = jest.fn();
const chunkAsset = jest.fn();
const processAssets = jest.fn();
const afterProcessAssets = jest.fn();
const seal = jest.fn();
const afterSeal = jest.fn();

class MyPlugin {
    apply(compiler) {
        compiler.hooks.compilation.tap("PLUGIN", compilation => {
            const localId = globalId += 1;

            compilation.hooks.buildModule.tap("PLUGIN", () => {
                buildModule();
                expect(localId).toBe(globalId);
            });
            compilation.hooks.succeedModule.tap("PLUGIN", () => {
                succeedModule();
                expect(localId).toBe(globalId);
            });
            compilation.hooks.finishModules.tap("PLUGIN", () => {
                finishModules();
                expect(localId).toBe(globalId);
            });
            compilation.hooks.optimizeModules.tap("PLUGIN", () => {
                optimizeModules();
                expect(localId).toBe(globalId);
            });
            compilation.hooks.afterOptimizeModules.tap("PLUGIN", () => {
                afterOptimizeModules();
                expect(localId).toBe(globalId);
            });
            compilation.hooks.optimizeTree.tap("PLUGIN", () => {
                optimizeTree();
                expect(localId).toBe(globalId);
            });
            compilation.hooks.optimizeChunkModules.tap("PLUGIN", () => {
                optimizeChunkModules();
                expect(localId).toBe(globalId);
            });
            compilation.hooks.additionalTreeRuntimeRequirements.tap("PLUGIN", () => {
                additionalTreeRuntimeRequirements();
                expect(localId).toBe(globalId);
            });
            compilation.hooks.runtimeModule.tap("PLUGIN", () => {
                runtimeModule();
                expect(localId).toBe(globalId);
            });
            compilation.hooks.chunkHash.tap("PLUGIN", () => {
                chunkHash();
                expect(localId).toBe(globalId);
            });
            compilation.hooks.chunkAsset.tap("PLUGIN", () => {
                chunkAsset();
                expect(localId).toBe(globalId);
            });
            compilation.hooks.processAssets.tap("PLUGIN", () => {
                processAssets();
                expect(localId).toBe(globalId);
            });
            compilation.hooks.afterProcessAssets.tap("PLUGIN", () => {
                afterProcessAssets();
                expect(localId).toBe(globalId);
            });
            compilation.hooks.seal.tap("PLUGIN", () => {
                seal();
                expect(localId).toBe(globalId);
            });
            compilation.hooks.afterSeal.tap("PLUGIN", () => {
                afterSeal();
                expect(localId).toBe(globalId);
            });
        });
    }
}

/** @type {import('../..').TCompilerCaseConfig} */
module.exports = {
    description: "The hooks should access the correct closure",
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
        expect(buildModule.mock.calls.length).toBeGreaterThanOrEqual(2);
        expect(succeedModule.mock.calls.length).toBeGreaterThanOrEqual(2);
        expect(finishModules.mock.calls.length).toBeGreaterThanOrEqual(2);
        expect(optimizeModules.mock.calls.length).toBeGreaterThanOrEqual(2);
        expect(afterOptimizeModules.mock.calls.length).toBeGreaterThanOrEqual(2);
        expect(optimizeTree.mock.calls.length).toBeGreaterThanOrEqual(2);
        expect(optimizeChunkModules.mock.calls.length).toBeGreaterThanOrEqual(2);
        expect(additionalTreeRuntimeRequirements.mock.calls.length).toBeGreaterThanOrEqual(2);
        expect(runtimeModule.mock.calls.length).toBeGreaterThanOrEqual(2);
        expect(chunkHash.mock.calls.length).toBeGreaterThanOrEqual(2);
        expect(chunkAsset.mock.calls.length).toBeGreaterThanOrEqual(2);
        expect(processAssets.mock.calls.length).toBeGreaterThanOrEqual(2);
        expect(afterProcessAssets.mock.calls.length).toBeGreaterThanOrEqual(2);
        expect(seal.mock.calls.length).toBeGreaterThanOrEqual(2);
        expect(afterSeal.mock.calls.length).toBeGreaterThanOrEqual(2);
    }
};
