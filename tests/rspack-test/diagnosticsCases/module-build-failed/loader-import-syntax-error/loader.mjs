/** @type {import('@rspack/test-tools').PitchLoaderDefinitionFunction} */
export default async function (content) {
	try {
		await this.importModule("./syntax-error.js");
	} catch (e) {
		expect(e).toBeDefined()
		return content;
	}
	// here should be unreachable
	throw new Error("unreachable");
};
