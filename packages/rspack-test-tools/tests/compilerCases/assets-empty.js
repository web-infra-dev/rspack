const { createFsFromVolume, Volume } = require("memfs");

class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Plugin", compilation => {
			let assets = compilation.getAssets();
			expect(assets.length).toBe(0);
			const { RawSource } = require("webpack-sources");
			compilation.emitAsset(
				"dd.js",
				new RawSource(`module.exports = "This is dd"`)
			);
			compilation.hooks.processAssets.tap("Plugin", assets => {
				let names = Object.keys(assets);

				expect(names.length).toBe(2); // ["main.js", "dd.js"]
				expect(names.includes("main.js")).toBeTruthy();
				expect(assets["main.js"].source().includes("This is d"));

				expect(names.includes("dd.js")).toBeTruthy();
			});
		});
	}
}

const outputFileSystem = createFsFromVolume(new Volume());
module.exports = {
	description: "should emit assets correctly",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()]
		};
	},
	async compiler(context, compiler) {
		compiler.outputFileSystem = outputFileSystem;
	},
	async check() {
		if (
			outputFileSystem.existsSync("/directory/main.js") &&
			outputFileSystem.existsSync("/directory/dd.js")
		) {
			const dd = outputFileSystem.readFileSync("/directory/dd.js", "utf-8");

			if (dd !== `module.exports="This is dd";`) {
				throw new Error("File content is not correct");
			}
		}
	}
};
