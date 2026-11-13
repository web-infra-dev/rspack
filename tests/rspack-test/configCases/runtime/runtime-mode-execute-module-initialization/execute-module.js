const calls = [];
__webpack_require__.i.push((options) => calls.push(options.id));
exports.value = require('./dependency');
exports.cached = __webpack_require__.c[module.id] === module;
exports.intercepted = calls.length;
exports.getPublicPath = () => __webpack_public_path__;
__webpack_public_path__ = 'from-module/';
