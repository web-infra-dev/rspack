const assert = require("assert");
const path = require("path");
const { NormalModule } = require("@rspack/core");

const PLUGIN_NAME = "BeforeLoadersPlugin";
const tagLoader = require.resolve("./tag-loader.js");
const noopLoader = require.resolve("./noop-loader.cjs");

class BeforeLoadersPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
			// Ident the configured options object is registered under, captured
			// from the hook so the assertion below does not hardcode a rule path.
			let configuredIdent;

			NormalModule.getCompilationHooks(compilation).beforeLoaders.tap(
				PLUGIN_NAME,
				(loaders, normalModule) => {
					const name = path.basename(normalModule.userRequest);

					if (name === "untouched.js") {
						// The list handed to the hook mirrors what `module.rules` resolved,
						// options object included.
						assert.strictEqual(loaders.length, 1);
						assert.strictEqual(loaders[0].loader, tagLoader);
						assert.deepStrictEqual(loaders[0].options, { tag: "+config" });
						assert.strictEqual(loaders[0].type, null);
						configuredIdent = loaders[0].ident;
						assert.ok(configuredIdent);
					}

					if (name === "add.js") {
						loaders.push({
							loader: tagLoader,
							options: { tag: "+added" },
							ident: null,
							type: null
						});
					}

					// The loader type comes from the loader itself, not from its
					// request, so a `.cjs` loader reports "commonjs".
					if (name === "typed.js") {
						assert.strictEqual(loaders.length, 1);
						assert.strictEqual(loaders[0].loader, noopLoader);
						assert.strictEqual(loaders[0].type, "commonjs");
					}

					if (name === "mutate.js") {
						loaders[0].options = { tag: "+mutated" };
					}

					// Rebuilding the entry instead of assigning to it drops the internal
					// snapshot but keeps the ident, which must still not be reused.
					if (name === "spread.js") {
						loaders[0] = { ...loaders[0], options: { tag: "+spread" } };
					}

					if (name === "remove.js") {
						loaders.length = 0;
					}
				}
			);

			compilation.hooks.finishModules.tap(PLUGIN_NAME, () => {
				// Rewriting a loader must not overwrite the options object every other
				// module matched by that rule receives.
				assert.deepStrictEqual(
					compiler.__internal__ruleSet.references.get(configuredIdent),
					{ tag: "+config" }
				);
			});
		});
	}
}

module.exports = BeforeLoadersPlugin;
