// maths.js

// This function isn't used anywhere, so
// Rollup excludes it from the bundle...
export function square(x) {
	return x * x;
}

// This function gets included
export function cube(x) {
	// rewrite this as `square( x ) * x`
	// and see what happens!
	return x * x * x;
}

export * as xxx from "./test.js";
