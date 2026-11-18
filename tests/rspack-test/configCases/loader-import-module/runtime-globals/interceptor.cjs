const calls = [];
const handler = options => {
	calls.push(options.id);
	options.module.exports.intercepted = true;
};

__webpack_require__.i.push(handler);
const first = require('./dependency.cjs');
const second = require('./dependency.cjs');
__webpack_require__.i.splice(__webpack_require__.i.indexOf(handler), 1);

module.exports = {
	intercepted: first.intercepted,
	calls: calls.length,
	cached: first === second,
};
