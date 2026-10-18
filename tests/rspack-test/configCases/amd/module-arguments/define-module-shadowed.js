const module = { value: "local" };

define(["module"], function (mod) {
	mod.exports = { shadow: module, mod };
});
