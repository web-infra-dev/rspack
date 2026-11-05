/** @type {import("@rspack/core").LoaderDefinition<{}, { shouldReplace: boolean }>} */
export default function (source) {
	if (this.shouldReplace) {
		this._module.buildInfo._isReplaced = true;
		// The rewritten entry additionally imports ./extra, so the rebuild
		// installs a dependency that was not in the original module.
		return `import extra from "./extra";\n${source}`;
	}
	return source;
}
