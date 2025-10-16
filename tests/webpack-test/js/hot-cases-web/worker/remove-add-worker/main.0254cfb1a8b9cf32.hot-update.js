"use strict";
self["webpackHotUpdate"]("main", {
"./compute.js": 
/*!********************!*\
  !*** ./compute.js ***!
  \********************/
(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  "default": () => (__WEBPACK_DEFAULT_EXPORT__)
});
if(Math.random() < 0) {
	new Worker(new URL(/* worker import */__webpack_require__.p + __webpack_require__.u("worker_js_1"), __webpack_require__.b));
}
/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (async () => {
	const worker = new Worker(new URL(/* worker import */__webpack_require__.p + __webpack_require__.u("worker_js"), __webpack_require__.b));
	const result = await new Promise((resolve, reject) => {
		worker.onmessage = ({ data }) => {
			if(typeof data === "string") {
				reject(new Error(data));
			} else {
				resolve(data);
			}
		};
		worker.postMessage("compute");
	});
	await worker.terminate();
	return result;
});


}),

},function(__webpack_require__) {
// webpack/runtime/get_full_hash
(() => {
__webpack_require__.h = () => ("0dfc997529b027fd")
})();

}
);