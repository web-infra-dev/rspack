// util module that compatible with webpack
// ```js
// const { NormalModuleReplacementPlugin, WebpackError, util } = compiler.webpack;
// ```

import { createHash } from "./createHash";

export default {
	get createHash() {
		return createHash;
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
