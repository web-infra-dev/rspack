exports.ids = ['main'];
exports.modules = {
"./child Lazy  recursive ^\.\/.*\.js$": function (module, exports, __webpack_require__) {
var map = {"./a.js": "./child/a.js","./b.js": "./child/b.js",};
function webpackAsyncContext(req) {
	if(!__webpack_require__.o(map, req)) {
		return Promise.resolve().then(function() {
			var e = new Error("Cannot find module '" + req + "'");
			e.code = 'MODULE_NOT_FOUND';
			throw e;
		});
	}
	// extract logic from generate
	var id = map[req];

	return __webpack_require__.el(id).then(function() {
		return __webpack_require__(id);
	});
}
webpackAsyncContext.keys = function() {
	return Object.keys(map);
};
webpackAsyncContext.id = "./child Lazy  recursive ^\.\/.*\.js$";
module.exports = webpackAsyncContext;},
"./index.js": function (module, exports, __webpack_require__) {
const request = 'a';
__webpack_require__.el("./child/a.js").then(__webpack_require__.bind(__webpack_require__, "./child/a.js")).then(__webpack_require__.ir).then(({ a  })=>console.log("Literal", a));
__webpack_require__.el("./child/b.js").then(__webpack_require__.bind(__webpack_require__, "./child/b.js")).then(__webpack_require__.ir).then(({ b  })=>console.log("Template Literal", b));
__webpack_require__('./child Lazy  recursive ^\.\/.*\.js$')(`./child/${request}.js`.replace("./child/", "./")).then(({ a  })=>console.log("context_module_tpl", a));
__webpack_require__('./child Lazy  recursive ^\.\/.*\.js$')(('./child/' + request + '.js').replace("./child/", "./")).then(({ a  })=>console.log("context_module_bin", a));
__webpack_require__('./child Lazy  recursive ^\.\/.*\.js$')("./child/".concat(request, ".js").replace("./child/", "./")).then(({ a  })=>console.log("context_module_concat", a));
},

};
var __webpack_require__ = require('./runtime.js');
__webpack_require__.C(exports)
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__('./index.js'));
