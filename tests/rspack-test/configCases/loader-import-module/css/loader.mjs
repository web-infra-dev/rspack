/** @type {import("@rspack/core").PitchLoaderDefinitionFunction} */
export const pitch = async function (remaining) {
	try {
		const result = await this.importModule(
			this.resourcePath + ".webpack[javascript/auto]" + "!=!" + remaining,
			this.getOptions()
		);
		return result.default || result;
	} catch (e) {
		console.error(e);
		throw e;
	}
};
