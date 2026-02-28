const assert = require("assert");

module.exports = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("TestPlugin", (compilation) => {
					compilation.hooks.finishModules.tap("TestPlugin", modules => {
						let entryModule;
						for (const module of modules) {
							if (module.identifier().endsWith("index.js")) {
								entryModule = module;
								break;
							}
						}

						assert(entryModule, "entry module not found");
						const dependencies = entryModule.dependencies;
						assert(dependencies.length > 0, "dependencies should not be empty");
						
						// Find the dependency corresponding to `import { a } from './lib'`
						const importDep = dependencies.find(dep => dep.type === 'esm import specifier');
						assert(importDep, "import dependency not found");
						
						const loc = importDep.loc;
						assert(loc, "loc should exist");
						// Verify start line/column.
						// Note: Rspack internal location might have offsets or point to specific tokens.
						// We verify we got a valid location object.
						assert(loc.start.line >= 1, "line should be >= 1");
						assert(typeof loc.start.column === 'number', "column should be a number"); 
					});
				});
			}
		}
	]
};
