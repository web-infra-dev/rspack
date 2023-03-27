// util module that compatible with webpack
// ```js
// const { NormalModuleReplacementPlugin, WebpackError, util } = compiler.webpack;
// ```

export default {
	get createHash() {
		return require("./createHash");
	}
	// get comparators() {
	// 	return require("./comparators");
	// },
	// get runtime() {
	// 	return require("./util/runtime");
	// },
	// get serialization() {
	// 	return require("./util/serialization");
	// },
	// get cleverMerge() {
	// 	return require("./util/cleverMerge").cachedCleverMerge;
	// },
	// get LazySet() {
	// 	return require("./util/LazySet");
	// }
};
