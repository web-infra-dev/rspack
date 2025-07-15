import { deepEqual, equal as eq } from "./assert";

export function myFunction() {
	function it() {
		eq(1, arguments[0]);
	}
	deepEqual(1, 1);
	it();
}

const bb = eq();
