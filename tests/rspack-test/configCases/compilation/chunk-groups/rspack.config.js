const PLUGIN_NAME = "plugin";

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
			compilation.hooks.afterProcessAssets.tap(PLUGIN_NAME, () => {
				const res = [];
				for (const chunkGroup of compilation.chunkGroups) {
					for (const origin of chunkGroup.origins) {
						res.push({
							request: origin.request,
							loc: origin.loc
						});
					}
				}
				res.sort((a, b) => a.request.localeCompare(b.request));
				expect(res).toMatchInlineSnapshot(`
					Array [
					  Object {
					    loc: Object {
					      end: Object {
					        column: 14,
					        line: 1,
					      },
					      start: Object {
					        column: 1,
					        line: 1,
					      },
					    },
					    request: ./a,
					  },
					  Object {
					    loc: Object {
					      end: Object {
					        column: 14,
					        line: 2,
					      },
					      start: Object {
					        column: 1,
					        line: 2,
					      },
					    },
					    request: ./b,
					  },
					  Object {
					    loc: main,
					    request: ./index.js,
					  },
					]
				`);
			});
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	plugins: [new Plugin()]
};
