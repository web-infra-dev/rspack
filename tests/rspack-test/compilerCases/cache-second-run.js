const { createFsFromVolume, Volume } = require("memfs");

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description: "should use cache on second watch compilation",
  options(context) {
    return {
      context: context.getSource(),
      entry: "./count-loader!./count-loader",
      devtool: false,
      mode: "development",
      output: {
        path: "/directory",
      },
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      let builds = 0;
      const watching = compiler.watch({}, (err) => {
        if (err) {
          return watching.close(() => reject(err));
        }
        if (builds++ === 0) {
          return watching.invalidate();
        }
        const result = compiler.outputFileSystem.readFileSync(
          "/directory/main.js",
          "utf-8"
        );
        try {
          expect(result).toContain("module.exports = 0;");
        } catch (error) {
          return watching.close(() => reject(error));
        }
        watching.close(resolve);
      });
    });
  },
};
