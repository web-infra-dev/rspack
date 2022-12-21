var scriptUrl;
// TODO: should use `__webpack_require__.g`
// if (__webpack_require__.g.importScripts) {
//   scriptUrl = __webpack_require__.g.location + ""
// };
// var document = __webpack_require__.g.document;
if (self.importScripts) {
	scriptUrl = self.location + "";
}
var document = self.document;
if (!scriptUrl && document) {
	if (document.currentScript) scriptUrl = document.currentScript.src;
	if (!scriptUrl) {
		var scripts = document.getElementsByTagName("script");
		if (scripts.length) scriptUrl = scripts[scripts.length - 1].src;
	}
}
// When supporting browsers where an automatic publicPath is not supported you must specify an output.publicPath manually via configuration
// or pass an empty string ("") and set the __webpack_public_path__ variable from your code to use your own logic.
if (!scriptUrl)
	throw new Error("Automatic publicPath is not supported in this browser");
scriptUrl = scriptUrl
	.replace(/#.*$/, "")
	.replace(/\?.*$/, "")
	.replace(/\/[^\/]+$/, "/");
__webpack_require__.p = scriptUrl;
