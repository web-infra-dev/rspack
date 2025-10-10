const { DefinePlugin, ProvidePlugin } = require("@rspack/core");

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
  description: "Should provide builtin plugins with correct class name",
  async build(context, compiler) { },
  async check({ context, stats, compiler, compilation }) {
    expect(new DefinePlugin({}).constructor.name).toEqual("DefinePlugin");
    expect(new ProvidePlugin({}).constructor.name).toEqual("ProvidePlugin");
  }
}];
