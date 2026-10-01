/** @type {import("@rspack/core").PitchLoaderDefinitionFunction} */
export default async function (remaining) {
	try {
		const result = await this.importModule("./index.png");
		expect(typeof result).toBe("string");
		const resultWithPublicPath = await this.importModule("./index.png", {
			publicPath: () => "/callback/"
		});
		expect(resultWithPublicPath).toBe(result.replace("/public/", "/callback/"));
		return `export default ${JSON.stringify(result)};`;
	} catch (e) {
		console.error(e);
		throw e;
	}
};
