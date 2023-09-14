(function (root, factory) {
	if (typeof exports === "object" && typeof module === "object")
		module.exports = factory();
	else if (typeof define === "function" && define.amd) define([], factory);
	else if (typeof exports === "object") exports["foo"] = factory();
	else root["foo"] = factory();
})(this, function () {
	return {
		data: "ok",
		default: "default"
	};
});
