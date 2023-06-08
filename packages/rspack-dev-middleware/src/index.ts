import wdm from "webpack-dev-middleware";
import util from "util";

/** @deprecated
 *
 * This package has the same functionality as webpack-dev-middleware, please use webpack-dev-middleware instead.
 */
const rdm: typeof wdm = util.deprecate((compiler, options) => {
	return wdm(compiler, options);
}, "@rspack/dev-middleware has the same functionality as webpack-dev-middleware, please use webpack-dev-middleware instead. This package will be removed in the next 'minor' release.");

export default rdm;
