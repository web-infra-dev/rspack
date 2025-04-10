import * as path from "path";
import { type Compilation, rspack, NormalModule } from "@rspack/core";
import { beforeAll, bench, describe } from "vitest";
import rspackConfig from "./fixtures/ts-react/rspack.config";

const BARREL_OPTIMIZATION_PREFIX = '__barrel_optimize__';

let context: string;
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
						context = compiler.context;

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

	bench("collect imported identifiers", () => {
		for (const [_, entry] of theCompilation.entries.entries()) {
			const entryDependency = entry.dependencies?.[0];
			if (!entryDependency || !entryDependency.request) continue;

			const entryModule = theCompilation.moduleGraph.getResolvedModule(entryDependency);
			if (!entryModule) continue;

			for (const connection of theCompilation.moduleGraph.getOutgoingConnectionsInOrder(entryModule)) {
				let importedIdentifiers: string[] = []
				if (connection.dependency?.ids) {
					importedIdentifiers.push(...connection.dependency.ids)
				} else {
					importedIdentifiers = ['*']
				}
			}
		}
	});

	bench("record module", () => {
		function recordModule(mod: NormalModule) {
			let resource =
				mod.type === 'css/mini-extract'
				? mod.identifier().slice(mod.identifier().lastIndexOf('!') + 1)
				: mod.resource
	
			if (!resource) {
				return
			}

			let ssrNamedModuleId = path.relative(
				context,
				mod.resourceResolveData?.path || resource
			);

			const rscNamedModuleId = path.relative(
				context,
				mod.resourceResolveData?.path || resource
			);

			const esmResource = /[\\/]next[\\/]dist[\\/]/.test(resource)
				? resource.replace(
					/[\\/]next[\\/]dist[\\/]/,
					'/next/dist/esm/'.replace(/\//g, path.sep)
				)
				: null

			if (mod.matchResource?.startsWith(BARREL_OPTIMIZATION_PREFIX)) {
			}
		}

		for (const module of theCompilation.modules) {
			if (module instanceof NormalModule) {
				recordModule(module);
			}
		}
	});

	bench("is css mod", () => {
		const regexCSS = /\.(css|scss|sass)(\?.*)?$/;

		function isCSSMod(mod: {
			resource: string
			type?: string
			loaders?: { loader: string }[]
		}): boolean {
			return !!(
			  	mod.type === 'css/mini-extract' ||
			  	(mod.resource && regexCSS.test(mod.resource)) ||
			 	mod.loaders?.some(
					({ loader }) =>
						loader.includes('next-style-loader/index.js') ||
						loader.includes('rspack.CssExtractRspackPlugin.loader') ||
						loader.includes('@vanilla-extract/webpack-plugin/loader/')
			  	)
			)
		}

		for (const module of theCompilation.modules) {
			if (module instanceof NormalModule) {
				isCSSMod(module);
			}
		}
	});
});
