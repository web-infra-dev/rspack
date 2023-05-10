(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./child Sync  recursive ^\.\/.*\.js$": function (module, exports, __webpack_require__) {
var map = {"./child/index.js": "./child/child/index.js","./index.js": "./child/index.js",};
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
webpackContext.id = './child Sync  recursive ^\.\/.*\.js$';

      webpackContext.keys = function webpackContextKeys() {
        return Object.keys(map);
      };
      webpackContext.resolve = webpackContextResolve;
      module.exports = webpackContext;
      },
"./child/child/index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "value", {
    enumerable: true,
    get: function() {
        return value;
    }
});
const value = "dynamic";
},
"./child/index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "value", {
    enumerable: true,
    get: function() {
        return value;
    }
});
const value = "dynamic";
},
"./index.js": function (module, exports, __webpack_require__) {
let a = "index";
__webpack_require__('./child Sync  recursive ^\.\/.*\.js$')(`./child/${a}.js`.replace("./child/", "./"));
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__('./index.js'));

}
]);