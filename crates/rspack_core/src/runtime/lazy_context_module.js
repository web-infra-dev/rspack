var map = $MAP$;
function webpackAsyncContext(req) {
	if(!__webpack_require__.o(map, req)) {
		return Promise.resolve().then(function() {
			var e = new Error("Cannot find module '" + req + "'");
			e.code = 'MODULE_NOT_FOUND';
			throw e;
		});
	}
	// extract logic from generate
	var ids = map[req];

	if(ids.length > 0) {
		return Promise.all(ids.map(__webpack_require__.e)).then(function() {
			return __webpack_require__(req);
		});
	} else {
		return Promise.resolve().then(function() {
			return __webpack_require__(req);
		});
	}
}
webpackAsyncContext.keys = function() {
	return Object.keys(map);
};
webpackAsyncContext.id = "$ID$";
module.exports = webpackAsyncContext;