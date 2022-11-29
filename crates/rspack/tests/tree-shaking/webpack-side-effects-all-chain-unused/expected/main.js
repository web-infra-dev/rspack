(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./../node_modules/pmodule/a.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "a", {
    enumerable: true,
    get: ()=>a
});
const _tracker = __webpack_require__("./../node_modules/pmodule/tracker.js");
var a = "a";
(0, _tracker.track)("a.js");
},
"./../node_modules/pmodule/index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__.exportStar(__webpack_require__("./../node_modules/pmodule/a.js"), exports);
const _tracker = __webpack_require__("./../node_modules/pmodule/tracker.js");
(0, _tracker.track)("index.js");
},
"./../node_modules/pmodule/tracker.js": function (module, exports, __webpack_require__) {
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
const _tracker = __webpack_require__("./../node_modules/pmodule/tracker.js");
const _pmodule = __webpack_require__("./../node_modules/pmodule/index.js");
_pmodule.a;
console.log(_tracker.log);
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);