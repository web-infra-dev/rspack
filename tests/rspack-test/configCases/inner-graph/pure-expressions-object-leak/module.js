import { a, b, c } from "dep";

// A PURE annotation attached to a preceding property's value must not leak
// into a later property's computed key check.
export const leak = {
	first: /*#__PURE__*/ Boolean(a),
	[Boolean(b)]: c
};
