import lib1 from "./lib1";
import lib2 from './lib2';
import lib3 from './lib3';

it("define(function (require, module, exports) {}) should work well", function () {
	expect(lib1.foo).toBe('foo');
	expect(lib1.bar).toBe('bar');

	expect(typeof lib1.add).toBe('function');
	expect(lib1.add(1, 2)).toBe(3);

	expect(typeof lib1.hello).toBe('function');
	expect(lib1.hello('world')).toBe('Hello, world');
});

it("should be able to specify require/module/exports in deps array", function () {
	expect(lib2.foo).toBe('foo');
	expect(lib2.bar).toBe('bar');

	expect(typeof lib2.add).toBe('function');
	expect(lib2.add(1, 2)).toBe(3);

	expect(typeof lib2.hello).toBe('function');
	expect(lib2.hello('world')).toBe('Hello, world');
});

it("should pass the exports object for the exports dependency", function () {
	// The factory parameter shadows the module exports argument, so the dependency
	// array must contain the module's own exports object.
	expect(lib3.sameAsModuleExports).toBe(true);
	expect(lib3.foo).toBe('foo');
	expect(lib3.bar).toBe('bar');
});
