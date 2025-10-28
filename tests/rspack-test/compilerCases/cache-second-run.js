const { createFsFromVolume, Volume } = require("memfs");

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description: "should use cache on second run call",
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
      compiler.run((err) => {
        compiler.run((err) => {
          const result = compiler.outputFileSystem.readFileSync(
            "/directory/main.js",
            "utf-8"
          );
          expect(result).toContain("module.exports = 0;");
          resolve();
        });
      });
    });
  },
};
