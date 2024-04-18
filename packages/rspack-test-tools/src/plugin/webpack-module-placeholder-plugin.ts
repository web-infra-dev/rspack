// @ts-nocheck
const {
	ConcatSource,
	RawSource,
	CachedSource,
	PrefixSource
} = require("webpack-sources");
const path = require("path");

function createRenderRuntimeModulesFn(Template) {
	return function renderRuntimeModules(runtimeModules, renderContext) {
		const source = new ConcatSource();
		for (const module of runtimeModules) {
			const codeGenerationResults = renderContext.codeGenerationResults;
			let runtimeSource;
			if (codeGenerationResults) {
				runtimeSource = codeGenerationResults.getSource(
					module,
					renderContext.chunk.runtime,
					"runtime"
				);
			} else {
				const codeGenResult = module.codeGeneration({
					chunkGraph: renderContext.chunkGraph,
					dependencyTemplates: renderContext.dependencyTemplates,
					moduleGraph: renderContext.moduleGraph,
					runtimeTemplate: renderContext.runtimeTemplate,
					runtime: renderContext.chunk.runtime,
					codeGenerationResults
				});
				if (!codeGenResult) continue;
				runtimeSource = codeGenResult.sources.get("runtime");
			}
			if (runtimeSource) {
				let identifier = module.identifier();
				source.add(Template.toNormalComment(`start::${identifier}`) + "\n");
				if (!module.shouldIsolate()) {
					source.add(runtimeSource);
					source.add("\n\n");
				} else if (renderContext.runtimeTemplate.supportsArrowFunction()) {
					source.add("(() => {\n");
					source.add(new PrefixSource("\t", runtimeSource));
					source.add("\n})();\n\n");
				} else {
					source.add("!function() {\n");
					source.add(new PrefixSource("\t", runtimeSource));
					source.add("\n}();\n\n");
				}
				source.add(Template.toNormalComment(`end::${identifier}`) + "\n");
			}
		}
		return source;
	};
}

const caches = new WeakMap();

export class WebpackModulePlaceholderPlugin {
	constructor() {}
	apply(compiler) {
		const { webpack } = compiler;
		const {
			Template,
			javascript: { JavascriptModulesPlugin }
		} = webpack;
		Template.renderRuntimeModules = createRenderRuntimeModulesFn(Template);
		compiler.hooks.compilation.tap("RuntimeDiffPlugin", compilation => {
			const hooks = JavascriptModulesPlugin.getCompilationHooks(compilation);
			hooks.inlineInRuntimeBailout.tap(
				"RuntimeDiffPlugin",
				() => "not allow inline startup"
			);
			hooks.renderModulePackage.tap(
				"RuntimeDiffPlugin",
				(moduleSource, module) => {
					let cacheEntry;
					let cache = caches.get(compilation);
					if (cache === undefined) {
						caches.set(compilation, (cache = new WeakMap()));
						cache.set(
							module,
							(cacheEntry = {
								header: undefined,
								footer: undefined,
								full: new WeakMap()
							})
						);
					} else {
						cacheEntry = cache.get(module);
						if (cacheEntry === undefined) {
							cache.set(
								module,
								(cacheEntry = {
									header: undefined,
									footer: undefined,
									full: new WeakMap()
								})
							);
						} else {
							const cachedSource = cacheEntry.full.get(moduleSource);
							if (cachedSource !== undefined) return cachedSource;
						}
					}
					const source = new ConcatSource();
					let header = cacheEntry.header;
					let footer = cacheEntry.footer;
					if (header === undefined) {
						const identifier = module.identifier();
						const moduleId = compilation.chunkGraph.getModuleId(module);
						header = new RawSource(
							`\n${Template.toNormalComment(`start::${moduleId}::${identifier}`)}\n`
						);
						footer = new RawSource(
							`\n${Template.toNormalComment(`end::${moduleId}::${identifier}`)}\n`
						);
						cacheEntry.header = header;
						cacheEntry.footer = footer;
					}
					source.add(header);
					source.add(moduleSource);
					source.add(footer);
					const cachedSource = new CachedSource(source);
					cacheEntry.full.set(moduleSource, cachedSource);
					return cachedSource;
				}
			);
		});
	}
}
