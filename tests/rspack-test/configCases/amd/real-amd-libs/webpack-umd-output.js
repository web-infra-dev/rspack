(function webpackUniversalModuleDefinition(root, factory) {
	if (typeof exports === 'object' && typeof module === 'object')
		module.exports = factory();
	else if (typeof define === 'function' && define.amd)
		define([], factory);
	else {
		var a = factory();
		for (var i in a) (typeof exports === 'object' ? exports : root)[i] = a[i];
	}
})(this, () => {
	return { version: '0.0.0' };
});
