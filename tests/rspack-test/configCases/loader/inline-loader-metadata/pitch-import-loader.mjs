export const pitch = function (request) {
	const callback = this.async();
	this.importModule(
		`${this.resourcePath}.rspack[javascript/auto]!=!!!${request}`,
		{},
		(err, exports) => {
			if (err) {
				callback(err);
				return;
			}
			callback(
				null,
				`module.exports = ${JSON.stringify(exports.default ?? exports)};`,
			);
		},
	);
};
