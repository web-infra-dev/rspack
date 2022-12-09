// var client = __webpack_require__("./hot/lazy-compilation-web.js?lazy-compilation-using-")
// var data = "$MODULE_ID$";
module.exports = __webpack_require__.l(
	"./lazy-compilation-web/$CHUNK_ID$",
	function () {}
); //.then(__webpack_require__.t.bind(__webpack_require__, "$MODULE_ID$", 23));
if (module.hot) {
	module.hot.accept();
	// module.hot.accept("$MODULE_ID$", function() { module.hot.invalidate(); });
	// module.hot.dispose(function(data) {
	//     delete data.resolveSelf;
	//     //dispose(data);
	// });
	// if (module.hot.data && module.hot.data.resolveSelf) module.hot.data.resolveSelf(module.exports);
}
function onError() {
	/* ignore */
}
// var dispose = client.keepAlive({ data: data, active: true, module: module, onError: onError });
