const { basename, defineCompileCase } = require("@rspack/test-tools");
const { createFsFromVolume, Volume } = require("memfs");


defineCompileCase(Utils.basename(__filename), {
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
});
