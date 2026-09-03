const assert = require("node:assert/strict");

function checkModuleLoaders(context) {
	assert.deepEqual(context._module.loaders.map(loader => loader.loader), [__filename]);
}

module.exports = function (content) {
	checkModuleLoaders(this);
	assert.equal(this.data.checkedModuleLoaders, true);
	return (
		"module.exports = " +
		JSON.stringify({
			resourcePath: this.resourcePath,
			prev: content
		})
	);
};

module.exports.pitch = async function () {
	checkModuleLoaders(this);
	await new Promise(resolve => setImmediate(resolve));
	checkModuleLoaders(this);
	this.data.checkedModuleLoaders = true;
};
