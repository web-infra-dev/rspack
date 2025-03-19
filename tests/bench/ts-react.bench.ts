import { type Compilation, rspack } from "@rspack/core";
import { beforeAll, bench, describe } from "vitest";
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
						compiler.hooks.compilation.tap("PLUGIN", compilation => {
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
	bench("Traverse module graph by dependencies", () => {
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

	bench("Traverse module graph by connections", () => {
		const entries = theCompilation.entries.values();

		const visitedModules = new Set();

		function traverse(connection) {
			const module = connection ? connection.module : null;
			if (module) {
				if (visitedModules.has(module)) {
					return;
				}
				const connections =
					theCompilation.moduleGraph.getOutgoingConnections(module);
				visitedModules.add(module);
				for (const c of connections) {
					traverse(c);
				}
			}
		}

		for (const entry of entries) {
			for (const dependency of entry.dependencies) {
				const connection = theCompilation.moduleGraph.getConnection(dependency);
				traverse(connection);
			}
		}
	});

	bench("Traverse compilation.modules", () => {
		for (const module of theCompilation.modules) {
			module.identifier();
		}
	});

	bench("stats.toJson()", () => {
		const json = theCompilation.getStats().toJson();
	});
});
