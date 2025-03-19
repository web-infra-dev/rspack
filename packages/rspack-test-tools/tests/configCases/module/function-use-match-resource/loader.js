module.exports = function (source) {
	const info = this.getOptions();
	delete info.compiler;
	return source + `
export const __info__ = ${JSON.stringify(info, null, 2)};
`
};
