export const __webpack_id__ = "main" ;
export const __webpack_ids__ = ["main"];
export const __webpack_modules__ = {
"./index.js": 
/*!******************!*\
  !*** ./index.js ***!
  \******************/
(function (module, __webpack_exports__, __webpack_require__) {
__webpack_require__.r(__webpack_exports__);
/* ESM import */var _shared__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./shared */ "./shared.js");
/* ESM import */var _update_esm__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ../../update.esm */ "../../update.esm.js");



it("should handle HMR with runtime chunk in ESM format", (done) => {
	expect(_shared__WEBPACK_IMPORTED_MODULE_0__.sharedData.version).toBe("1.0.0");
	
	module.hot.accept([/*! ./shared */ "./shared.js"], function(){
/* ESM import */_shared__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./shared */ "./shared.js");

});
	
	NEXT((0,_update_esm__WEBPACK_IMPORTED_MODULE_1__["default"])(done, true, () => {
		Promise.resolve(/*! import() */).then(__webpack_require__.bind(__webpack_require__, /*! ./shared */ "./shared.js")).then(updatedModule => {
			expect(updatedModule.sharedData.version).toBe("2.0.0");
			done();
		}).catch(done);
	}));
});

it("should load async shared module with runtime chunk", (done) => {
	__webpack_require__.e(/*! import() */ "async-shared_js").then(__webpack_require__.bind(__webpack_require__, /*! ./async-shared */ "./async-shared.js")).then(module => {
		expect(module.asyncData.loaded).toBe(true);
		expect(module.asyncData.content).toBe("Async shared content");
		done();
	}).catch(done);
});


}),
"./shared.js": 
/*!*******************!*\
  !*** ./shared.js ***!
  \*******************/
(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  sharedData: () => (sharedData)
});
const sharedData = {
	version: "2.0.0",
	timestamp: Date.now()
};


}),
"../../update.esm.js": 
/*!***************************!*\
  !*** ../../update.esm.js ***!
  \***************************/
(function (module, __webpack_exports__, __webpack_require__) {
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  "default": () => (update)
});
function update(done, options, callback) {
	return function (err, stats) {
		if (err) return done(err);
		module.hot
			.check(options || true)
			.then(updatedModules => {
				if (!updatedModules) {
					return done(new Error("No update available"));
				}
				if (callback) callback(stats);
			})
			.catch(err => {
				done(err);
			});
	};
};

}),

};
import __webpack_require__ from './runtime.mjs';
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId); }
import * as __webpack_chunk_$1__ from './main.mjs';
__webpack_require__.C(__webpack_chunk_$1__);
var __webpack_exports__ = __webpack_exec__("./index.js");