define(function (require, exports, module) {
	const { foo, bar } = require('./constants');
	exports.foo = foo;
	exports.bar = bar;
	exports.add = require('./add');
	exports.hello = require('./hello');
});
