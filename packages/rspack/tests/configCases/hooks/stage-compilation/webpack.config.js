const TestPlugin = require("./plugin")

module.exports = {
	plugins: [new TestPlugin((compiler, list) => {
		const pushBanner = (compiler, banner, tapOptions) => {
			compiler.hooks.compilation.tap(tapOptions, compilation => {
				list.push(`/* ${banner} */`);
			});
		}
		pushBanner(compiler, "banner1", { name: "banner1", stage: 100 });
		pushBanner(compiler, "banner2", {
			name: "banner2",
			before: "banner1"
		});
		pushBanner(compiler, "banner3", { name: "banner3", stage: -100 });
		pushBanner(compiler, "banner4", { name: "banner4" });
		pushBanner(compiler, "banner5", { name: "banner5", stage: -Infinity });
		pushBanner(compiler, "banner6", { name: "banner6", stage: Infinity });
	})]
};
