// eslint-disable-next-line node/no-unpublished-require
const { ProvideSharedPlugin } = require("@rspack/core").sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ProvideSharedPlugin({
			provides: ["x", "y"]
		}),
    function (compiler) {
      compiler.hooks.thisCompilation.tap(
        "customResolver",
        (compilation, { normalModuleFactory }) => {
          normalModuleFactory.hooks.resolve.tap("customResolver", (data) => {
            if (data.request === "x") {
              data.request = require.resolve(data.request);
            }
          });
        }
			)
		}
	]
};
