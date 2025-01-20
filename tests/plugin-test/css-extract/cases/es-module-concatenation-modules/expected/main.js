(() => { // webpackBootstrap
"use strict";
// The require scope
var __webpack_require__ = {};

/************************************************************************/
// webpack/runtime/define_property_getters
(() => {
__webpack_require__.d = (exports, definition) => {
	for(var key in definition) {
        if(__webpack_require__.o(definition, key) && !__webpack_require__.o(exports, key)) {
            Object.defineProperty(exports, key, { enumerable: true, get: definition[key] });
        }
    }
};
})();
// webpack/runtime/has_own_property
(() => {
__webpack_require__.o = (obj, prop) => (Object.prototype.hasOwnProperty.call(obj, prop))
})();
// webpack/runtime/make_namespace_object
(() => {
// define __esModule on exports
__webpack_require__.r = (exports) => {
	if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
		Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
	}
	Object.defineProperty(exports, '__esModule', { value: true });
};

})();
/************************************************************************/
var __webpack_exports__ = {};
// ESM COMPAT FLAG
__webpack_require__.r(__webpack_exports__);

// EXPORTS
__webpack_require__.d(__webpack_exports__, {
  b: () => (/* reexport */ b_namespaceObject),
  a: () => (/* reexport */ a_namespaceObject),
  c: () => (/* reexport */ c_1)
});
// NAMESPACE OBJECT: ./a.css
var a_namespaceObject = {};
__webpack_require__.r(a_namespaceObject);
__webpack_require__.d(a_namespaceObject, { 
  a: () => (_1) });

// NAMESPACE OBJECT: ./b.css
var b_namespaceObject = {};
__webpack_require__.r(b_namespaceObject);
__webpack_require__.d(b_namespaceObject, { 
  b: () => (b_1) });

// NAMESPACE OBJECT: ./index.js
var index_namespaceObject = {};
__webpack_require__.r(index_namespaceObject);
__webpack_require__.d(index_namespaceObject, { 
  a: () => (a_namespaceObject),
  b: () => (b_namespaceObject),
  c: () => (c_1) });


;// CONCATENATED MODULE: ./a.css
// extracted by css-extract-rspack-plugin
var _1 = "foo__a";


;// CONCATENATED MODULE: ./b.css
// extracted by css-extract-rspack-plugin
var b_1 = "foo__b";


;// CONCATENATED MODULE: ./c.css
// extracted by css-extract-rspack-plugin
var c_1 = "foo__c";


;// CONCATENATED MODULE: ./index.js
/* eslint-disable import/no-namespace */








// eslint-disable-next-line no-console
console.log(index_namespaceObject);

})()
;