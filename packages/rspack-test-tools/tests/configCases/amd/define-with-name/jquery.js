
define('foo', 'foo');

define('add', function () {
	return function add(a, b) {
		return a + b;
	};
});

define('jQuery', [], function () {
	return 'jQuery';
});
