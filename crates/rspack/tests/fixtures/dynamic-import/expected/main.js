(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
    "./. Lazy  recursive ^.*.js$": function (module, exports, __webpack_require__) {
    var map = {"./a.js": "./a.js","./b.js": "./b.js","./c.js": "./c.js","./expected/main.js": "./expected/main.js","./index.js": "./index.js",};
    function webpackAsyncContext(req) {
        if(!__webpack_require__.o(map, req)) {
            return Promise.resolve().then(function() {
                var e = new Error("Cannot find module '" + req + "'");
                e.code = 'MODULE_NOT_FOUND';
                throw e;
            });
        }
        // extract logic from generate
        var id = map[req];
    
        return __webpack_require__.el(id).then(function() {
            return __webpack_require__(id);
        });
    }
    webpackAsyncContext.keys = function() {
        return Object.keys(map);
    };
    webpackAsyncContext.id = "./. Lazy  recursive ^.*.js$";
    module.exports = webpackAsyncContext;},
    "./index.js": function (module, exports, __webpack_require__) {
    const request = 'c';
    __webpack_require__.el("./a.js").then(__webpack_require__.bind(__webpack_require__, "./a.js")).then(__webpack_require__.ir).then(({ a  })=>console.log(a));
    __webpack_require__.el("./b.js").then(__webpack_require__.bind(__webpack_require__, "./b.js")).then(__webpack_require__.ir).then(({ b  })=>console.log(b));
    __webpack_require__('./. Lazy  recursive ^.*.js$')(`./${request}.js`).then(({ c  })=>console.log(c));
    },
    
    },function(__webpack_require__) {
    var __webpack_exports__ = __webpack_require__('./index.js');
    
    }
    ]);