(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"../node_modules/pmodule/a.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'a': function() { return a; }});
var a = "a";
var b = "b";
var c = "c";


track("a.js");
},
"../node_modules/pmodule/b.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'x': function() { return x; }});
var x = "x";
var y = "y";



track("b.js");
},
"../node_modules/pmodule/c.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'z': function() { return z; }});
var z = "z";


track("c.js");
},
"../node_modules/pmodule/index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.es(_a__WEBPACK_IMPORTED_MODULE__, exports);



track("index.js");
var __WEBPACK_DEFAULT_EXPORT__ = "def";
},
"../node_modules/pmodule/tracker.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'track': function() { return track; }, 'log': function() { return log; }});
 function track(file) {
    log.push(file);
    log.sort();
}
 var log = [];
 function reset() {
    log.length = 0;
}
},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var pmodule__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* pmodule */"../node_modules/pmodule/index.js");



def.should.be.eql("def");
a.should.be.eql("a");
x.should.be.eql("x");
z.should.be.eql("z");
log.should.be.eql([
    "a.js",
    "b.js",
    "c.js",
    "index.js"
]);
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);