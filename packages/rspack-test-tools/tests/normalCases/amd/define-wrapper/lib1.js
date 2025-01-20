define(function (module, exports, require) {
	const { foo, bar } = require('./constants');
	exports.foo = foo;
	exports.bar = bar;
	exports.add = require('./add');
	exports.hello = require('./hello');
});
