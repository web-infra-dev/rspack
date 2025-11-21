const { createFsFromVolume, Volume } = require("memfs");
const failedSpy = rstest.fn();

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
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
};
