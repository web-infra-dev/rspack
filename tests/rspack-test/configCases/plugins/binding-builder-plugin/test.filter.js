const enableBindingTesting = !!process.env.RSPACK_BINDING;

module.exports = function (config) {
	return enableBindingTesting
};