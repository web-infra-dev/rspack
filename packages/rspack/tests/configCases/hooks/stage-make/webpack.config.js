const TestPlugin = require("../stage-compilation/plugin");

module.exports = {
	plugins: [new TestPlugin((compiler, list) => {
		const pushBanner = (compiler, banner, tapOptions) => {
			compiler.hooks.make.tap(tapOptions, () => {
				list.push(`/* sync ${banner} */`);
			});
			compiler.hooks.make.tapAsync(tapOptions, (compilation, callback) => {
				list.push(`/* async ${banner} */`);
				callback()
			});
			compiler.hooks.make.tapPromise(tapOptions, async () => {
				list.push(`/* promise ${banner} */`);
			});
		}
		pushBanner(compiler, "banner1", { name: "banner1", stage: 100 });
		pushBanner(compiler, "banner2", {
			name: "banner2",
			before: "banner1"
		});
		pushBanner(compiler, "banner3", { name: "banner3", stage: -100 });
		pushBanner(compiler, "banner4", { name: "banner4" });
	})]
};
