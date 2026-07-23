const fs = require("fs");
const path = require("path");

module.exports = {
	findBundle: () => [],
	validate(stats, _stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const statsJson = stats.toJson({
			modules: true,
			nestedModules: true
		});
		const modules =
			statsJson.modules ??
			statsJson.children?.flatMap(child => child.modules ?? []) ??
			[];
		const concatenatedModule = modules.find(module => {
			const innerIdentifiers =
				module.modules?.map(innerModule => innerModule.identifier) ?? [];
			return ["index.mjs", "bar.mjs"].every(filename =>
				innerIdentifiers.some(identifier => identifier.endsWith(filename))
			);
		});
		expect(concatenatedModule).toBeDefined();

		const manifest = JSON.parse(
			fs.readFileSync(path.resolve(config.output.path, "manifest.json"), "utf-8")
		);

		expect(Object.keys(manifest.content)).toContain(
			"./configCases/dll-plugin/concatenated-lib-ident-context/index.mjs"
		);
	}
};
