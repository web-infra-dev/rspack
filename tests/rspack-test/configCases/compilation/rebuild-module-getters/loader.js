let count = 0;
module.exports = function (source) {
	count++;
	if (count === 1) {
		return source;
	}
	let blocksError = '';
	let dependenciesError = '';
	try {
		this._module.blocks;
	} catch (e) {
		blocksError = String((e && e.message) || e);
	}
	try {
		this._module.dependencies;
	} catch (e) {
		dependenciesError = String((e && e.message) || e);
	}
	return (
		source +
		`\nexport const blocksError = ${JSON.stringify(blocksError)};` +
		`\nexport const dependenciesError = ${JSON.stringify(dependenciesError)};`
	);
};
