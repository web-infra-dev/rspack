define('foo', 'foo');
define('bar', 'bar');

define('add', function () {
	return (a, b) => a + b;
});

define('hello', function () {
	return function (name) {
		console.log(`Hello, ${name}`);
	};
});

define(['foo', 'add'], function (foo, add) {
	return { foo, add };
});
