const assert = require("node:assert/strict");

module.exports = function () {
	const originalFactoryMeta = this._module.factoryMeta;
	this._module.factoryMeta = { sideEffectFree: false };
	assert.equal(this._module.factoryMeta.sideEffectFree, false);
	assert.equal(originalFactoryMeta.sideEffectFree, true);
	this._module.factoryMeta = {};
	assert.equal(this._module.factoryMeta.sideEffectFree, undefined);
	this._module.factoryMeta = originalFactoryMeta;

	const assets = this._module.buildInfo.assets;
	assert.deepEqual(Object.keys(assets), []);
	this.emitFile("metadata.txt", "metadata");
	assert.equal(this._module.buildInfo.assets, assets);
	assert.deepEqual(Object.keys(assets), ["metadata.txt"]);

	this._module.buildInfo.LOADER_ACCESS = true;
	this._module.buildMeta.LOADER_ACCESS = true;
	return (
		"module.exports = " +
		JSON.stringify({
			request: this._module.request,
			userRequest: this._module.userRequest,
			rawRequest: this._module.rawRequest,
		})
	);
};
