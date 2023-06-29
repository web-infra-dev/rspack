(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
__webpack_require__(/* ./resources */"./resources Sync  recursive ^\\.\\/pre_.*\\.js$")((`./resources/pre_${i + 1}.js`).replace('./resources/', './'));
},
"./resources/pre_a.js": function (module, exports, __webpack_require__) {
console.log('a');
},
"./resources/pre_b.js": function (module, exports, __webpack_require__) {
console.log('a');
},
"./resources/pre_c.js": function (module, exports, __webpack_require__) {
console.log('a');
},
"./resources Sync  recursive ^\\.\\/pre_.*\\.js$": function (module, exports, __webpack_require__) {
var map = {"./pre_a.js": "./resources/pre_a.js","./pre_b.js": "./resources/pre_b.js","./pre_c.js": "./resources/pre_c.js",};
function webpackContext(req) {
var id = webpackContextResolve(req);

return __webpack_require__(id);

}
function webpackContextResolve(req) {

      if(!__webpack_require__.o(map, req)) {
        var e = new Error("Cannot find module '" + req + "'");
        e.code = 'MODULE_NOT_FOUND';
        throw e;
      }
      return map[req];
    
}
webpackContext.id = '"./resources Sync  recursive ^\\.\\/pre_.*\\.js$"';

      webpackContext.keys = function webpackContextKeys() {
        return Object.keys(map);
      };
      webpackContext.resolve = webpackContextResolve;
      module.exports = webpackContext;
      },

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);