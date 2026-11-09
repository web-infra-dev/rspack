export default async function () {
	const { default: value } = await this.importModule(this.resourcePath);
	return `export default ${JSON.stringify(value)};`;
}
