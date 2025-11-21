import * as lib from './lib';

it("define([...], function () {...}) should work well", function () {
	expect(lib.FOO).toBe('foo');
	expect(typeof lib.add).toBe('function');
	expect(lib.add(1, 2)).toBe(3);
	expect(typeof lib.hello).toBe('function');
	expect(lib.hello('foo')).toBe('Hello, foo');
});
