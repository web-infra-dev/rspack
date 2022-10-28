self["__rspack_runtime__"].__rspack_register__(["main"], {
"./answer.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "answer", {
    enumerable: true,
    get: ()=>answer
});
const answer = 42;
},
"./app.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "render", {
    enumerable: true,
    get: ()=>render
});
const _lib = __rspack_require__("./lib.js");
console.log('answer:', _lib.myanswer, _lib.secret);
function render() {
    const container = document.getElementById('root');
    container.innerHTML = `secret:${_lib.secret}\nanswer:${_lib.myanswer}`;
}
if (module.hot?.accept) module.hot.accept((module1)=>{
    console.log('xxx:', module1);
    render();
});
},
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _app = __rspack_require__("./app.js");
(0, _app.render)();
},
"./lib.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
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
    secret: ()=>secret,
    myanswer: ()=>myanswer
});
const _answer = __rspack_require__("./answer.js");
const secret = '24';
const myanswer = _answer.answer;
},
});self["__rspack_runtime__"].__rspack_require__("./index.js");