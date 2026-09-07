const assert = require("node:assert/strict");

module.exports = function () {
	const originalFactoryMeta = this._module.factoryMeta;
	this._module.factoryMeta = { sideEffectFree: false };
	assert.equal(this._module.factoryMeta.sideEffectFree, false);
	assert.equal(originalFactoryMeta.sideEffectFree, true);
	this._module.factoryMeta = {};
	assert.equal(this._module.factoryMeta.sideEffectFree, undefined);
	this._module.factoryMeta = originalFactoryMeta;

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
