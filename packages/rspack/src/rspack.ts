/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

// deliberately alias rspack as webpack
export {
	MultiStats,
	Stats,
	createCompiler,
	createMultiCompiler,
	rspack
} from "./Compiler";
