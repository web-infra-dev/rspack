import * as lib from "./lib";

import jQuery from './jquery';

it("should be able to define a local module with a name", function () {
	expect(lib.foo).toBe('foo');
	expect(typeof lib.add).toBe('function');
	expect(lib.add(1, 2)).toBe(3);
});

it('should export last unused local module', function () {
	expect(jQuery).toBe('jQuery');
});
