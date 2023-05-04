(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"../node_modules/pmodule/a.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "a", {
    enumerable: true,
    get: function() {
        return a;
    }
});
var _tracker = __webpack_require__("../node_modules/pmodule/tracker.js");
var a = "a";
(0, _tracker.track)("a.js");
},
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
    x: function() {
        return x;
    },
    z: function() {
        return _c.z;
    }
});
var _c = __webpack_require__("../node_modules/pmodule/c.js");
var _tracker = __webpack_require__("../node_modules/pmodule/tracker.js");
var x = "x";
(0, _tracker.track)("b.js");
},
"../node_modules/pmodule/c.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "z", {
    enumerable: true,
    get: function() {
        return z;
    }
});
var _tracker = __webpack_require__("../node_modules/pmodule/tracker.js");
var z = "z";
(0, _tracker.track)("c.js");
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
    x: function() {
        return _b.x;
    },
    z: function() {
        return _b.z;
    },
    default: function() {
        return _default;
    }
});
__webpack_require__.es(__webpack_require__("../node_modules/pmodule/a.js"), exports);
var _b = __webpack_require__("../node_modules/pmodule/b.js");
var _tracker = __webpack_require__("../node_modules/pmodule/tracker.js");
(0, _tracker.track)("index.js");
var _default = "def";
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
    track: function() {
        return track;
    },
    log: function() {
        return log;
    }
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
var _tracker = __webpack_require__("../node_modules/pmodule/tracker.js");
var _index = __webpack_require__.ir(__webpack_require__("../node_modules/pmodule/index.js"));
_index.default.should.be.eql("def");
_index.a.should.be.eql("a");
_index.x.should.be.eql("x");
_index.z.should.be.eql("z");
_tracker.log.should.be.eql([
    "a.js",
    "b.js",
    "c.js",
    "index.js"
]);
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__('./index.js'));

}
]);