module.exports = function () {
	globalThis.pureRequireCalls++;
};
module.exports.value = 43;
module.exports.effect = function () {
	globalThis.pureRequireCalls++;
};
