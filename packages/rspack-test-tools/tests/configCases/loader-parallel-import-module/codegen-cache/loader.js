/** @type {import("../../../../").PitchLoaderDefinitionFunction} */
module.exports = async function (remaining) {
	try {
		const result = await this.importModule("./app.js", this.getOptions());
		return `export default ${result.default}`;
	} catch (e) {
		console.error(e);
		throw e;
	}
};
