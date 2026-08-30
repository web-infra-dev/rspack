import { x, y } from "./dependency";

// `x` is referenced inside an inline `define` callback. Without the fix
// for issue #17063, the arguments of `define(...)` calls in ES modules were
// not walked, so the reference to `x` was not tracked.
function useX() {
	define(function () {
		return x;
	});
}

const callback = function () {
	return y;
};

function useY() {
	define(callback);
}

export { useX, useY };
