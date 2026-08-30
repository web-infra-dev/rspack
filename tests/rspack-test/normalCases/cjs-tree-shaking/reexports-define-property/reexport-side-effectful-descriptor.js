const counter = require("./counter");

Object.defineProperty(exports, "value", {
	enumerable: (counter.value++, true),
	value: require("./module?side-effectful-descriptor").abc
});
