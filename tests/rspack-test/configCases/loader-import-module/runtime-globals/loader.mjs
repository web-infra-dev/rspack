export default async function () {
	const result = await this.importModule(this.resourcePath, {});
	return `module.exports = ${JSON.stringify(result)};`;
}
