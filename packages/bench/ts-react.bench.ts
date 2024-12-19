import { rspack, Compilation } from "@rspack/core";
import { bench, describe, beforeAll } from "vitest";
import rspackConfig from "./fixtures/ts-react/rspack.config";

let theCompilation: Compilation;

beforeAll(() => {
	return new Promise((resolve, reject) =>
		rspack(
			{
				...rspackConfig,
				mode: "production",
				plugins: [
					...(rspackConfig.plugins ?? []),
					compiler => {
						compiler.hooks.compilation.tap("PLGUIN", compilation => {
							theCompilation = compilation;
						});
					}
				]
			},
			(err, stats) => {
				if (err) {
					reject(err);
				}
				if (stats?.hasErrors()) {
					reject(new Error(stats.toString({})));
				}
				resolve(undefined);
			}
		)
	);
});

describe("TypeScript React project", () => {
	bench("Traverse chunk modules", () => {
		const entries = theCompilation.entries.values();

		const visitedModules = new Set();

		function traverse(dependency) {
			const module = theCompilation.moduleGraph.getModule(dependency);
			if (module) {
				if (visitedModules.has(module)) {
					return;
				}
				visitedModules.add(module);
				for (const dep of module.dependencies) {
					traverse(dep);
				}
			}
		}

		for (const entry of entries) {
			for (const dependency of entry.dependencies) {
				traverse(dependency);
			}
		}
	});
});
