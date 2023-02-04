(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"../node_modules/pmodule/b.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    x: ()=>x,
    z: ()=>_cJs.z
});
const _cJs = __webpack_require__("../node_modules/pmodule/c.js");
const _trackerJs = __webpack_require__("../node_modules/pmodule/tracker.js");
var x = "x";
(0, _trackerJs.track)("b.js");
},
"../node_modules/pmodule/c.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "z", {
    enumerable: true,
    get: ()=>z
});
const _trackerJs = __webpack_require__("../node_modules/pmodule/tracker.js");
var z = "z";
(0, _trackerJs.track)("c.js");
},
"../node_modules/pmodule/index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    x: ()=>_bJs.x,
    z: ()=>_bJs.z,
    default: ()=>_default
});
const _bJs = __webpack_require__("../node_modules/pmodule/b.js");
const _trackerJs = __webpack_require__("../node_modules/pmodule/tracker.js");
(0, _trackerJs.track)("index.js");
const _default = "def";
},
"../node_modules/pmodule/tracker.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    track: ()=>track,
    log: ()=>log
});
function track(file) {
    log.push(file);
    log.sort();
}
var log = [];
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _trackerJs = __webpack_require__("../node_modules/pmodule/tracker.js");
const _indexJs = __webpack_require__.ir(__webpack_require__("../node_modules/pmodule/index.js"));
_indexJs.default.should.be.eql("def");
_indexJs.a.should.be.eql("a");
_indexJs.x.should.be.eql("x");
_indexJs.z.should.be.eql("z");
_trackerJs.log.should.be.eql([
    "a.js",
    "b.js",
    "c.js",
    "index.js"
]);
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);