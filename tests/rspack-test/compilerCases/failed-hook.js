const { basename, defineCompileCase } = require("@rspack/test-tools");
const { createFsFromVolume, Volume } = require("memfs");
const failedSpy = jest.fn();


defineCompileCase(Utils.basename(__filename), {
  description: "should call the failed-hook on error",
  options(context) {
    return {
      entry: "./missing",
      bail: true,
    };
  },
  async compiler(context, compiler) {
    compiler.hooks.failed.tap("CompilerTest", failedSpy);
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.run((err, stats) => {
        expect(err).toBeTruthy();
        expect(failedSpy).toHaveBeenCalledTimes(1);
        expect(failedSpy).toHaveBeenCalledWith(err);
        resolve();
      });
    });
  },
});
