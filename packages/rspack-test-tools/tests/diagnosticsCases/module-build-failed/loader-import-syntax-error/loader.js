/** @type {import("../../../../dist").PitchLoaderDefinitionFunction} */
module.exports = async function () {
	try {
		const result = await this.importModule("./syntax-error.js");

		// here should be unreachable
		expect(result).toBe(Symbol('unreachable'));
	} catch (e) {
		expect(e).toBeDefined()
	}
};
