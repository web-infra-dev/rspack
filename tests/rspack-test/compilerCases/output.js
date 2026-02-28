const rspack = require("@rspack/core");
const { createFsFromVolume, Volume } = require("memfs");
const outputFileSystem = createFsFromVolume(new Volume());

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
  description: "should be cleared the build directory(before)",
  options(context) {
    const options = rspack.config.getNormalizedRspackOptions({
      output: {
        path: context.getDist("../output"),
        filename: "hell1.js"
      }
    });
    if (!options.mode) options.mode = "production";
    options.entry = "./a";
    options.context = context.getSource();
    options.optimization = {
      minimize: false
    };
    options.cache = true;
    return options;
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = outputFileSystem;
    compiler.hooks.compilation.tap("CompilerTest", compilation => {
      compilation.bail = true;
    });
  },
  async check({ context, stats, compiler, compilation }) {
    expect(typeof stats).toBe("object");
    expect(stats).toHaveProperty("errors");
    expect(Array.isArray(stats.errors)).toBe(true);
    if (stats.errors.length > 0) {
      expect(stats.errors[0]).toBeInstanceOf(Error);
      throw stats.errors[0];
    }
  }
}, {
  description: "should be cleared the build directory(after)",
  options(context) {
    const options = rspack.config.getNormalizedRspackOptions({
      output: {
        clean: true,
        path: context.getDist("../output"),
        filename: "hell2.js"
      }
    });
    if (!options.mode) options.mode = "production";
    options.entry = "./a";
    options.context = context.getSource();
    options.optimization = {
      minimize: false
    };
    options.cache = true;
    return options;
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = outputFileSystem;
    compiler.hooks.compilation.tap("CompilerTest", compilation => {
      compilation.bail = true;
    });
  },
  async check({ context, stats, compiler, compilation }) {
    expect(typeof stats).toBe("object");
    expect(stats).toHaveProperty("errors");
    expect(Array.isArray(stats.errors)).toBe(true);
    if (stats.errors.length > 0) {
      expect(stats.errors[0]).toBeInstanceOf(Error);
      throw stats.errors[0];
    }
    expect(outputFileSystem.readdirSync(context.getDist("../output"))).toEqual(["hell2.js"]);
  }
}];
