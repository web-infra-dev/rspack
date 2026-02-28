
define(function (require) {
	const { HELLO } = require('./constants');

	return function hello(name) {
		return `${HELLO}, ${name}`;
	};
});
