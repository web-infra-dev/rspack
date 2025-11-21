module.exports = async function () {
	const [exports_1, exports_2] = await Promise.all([
		this.importModule(this.resourcePath, {}),
		this.importModule(this.resourcePath, {})
	]);

	const value = exports_1.default + exports_2.default;
	return `export default ${value}`;
};
