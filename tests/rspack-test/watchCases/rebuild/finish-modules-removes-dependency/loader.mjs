/** @type {import("@rspack/core").LoaderDefinition<{}, { shouldReplace: boolean }>} */
export default function (source) {
	if (this.shouldReplace) {
		this._module.buildInfo._isReplaced = true;
		// The rewritten entry drops the import of ./dep, which orphaned the
		// module behind it; the assertions must survive the rewrite.
		return `
			it("should drop a dependency during rebuildModule in finishModules", function() {
				expect(
					__STATS__.modules
						.filter(m => m.moduleType !== "runtime")
						.map(m => m.name)
						.sort()
				).toEqual(["./index.js"]);
			});
		`;
	}
	return source;
}
