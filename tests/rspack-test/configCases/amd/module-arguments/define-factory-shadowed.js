const module = { value: "local" };

define(function (require, exports, mod) {
	return { value: 42, shadow: module, mod };
});
