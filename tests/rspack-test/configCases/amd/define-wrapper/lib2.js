define(['./constants', 'exports', 'require'], function (constants, e, r) {
	e.foo = constants.foo;
	e.bar = constants.bar;
	e.add = r('./add');
	e.hello = r('./hello');
});
