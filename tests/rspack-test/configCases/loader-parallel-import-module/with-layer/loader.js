/** @type {import("@rspack/core").PitchLoaderDefinitionFunction} */
module.exports = async function (remaining) {
	try {
		const result = await this.importModule("./banner.js", {
			layer: "loader"
		});
		expect(result).toEqual("data");
		return `export default ${result}`;
	} catch (e) {
		console.error(e);
		throw e;
	}
};
