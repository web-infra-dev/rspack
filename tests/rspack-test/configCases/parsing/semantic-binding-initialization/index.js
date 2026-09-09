import { read, imported } from "./imports";

it("keeps import metadata after initializing lexical bindings", () => {
	expect(read()).toBe("dependency");
	expect(imported).toBe("dependency");
});

it("initializes all parameters before walking their defaults", () => {
	function later(get = () => require("./missing-parameter"), require = () => "parameter") {
		return get();
	}
	function separate(get = () => require("./value")) {
		var require = () => "body";
		return [get(), require("./missing-body")];
	}
	expect(later()).toBe("parameter");
	expect(separate()).toEqual(["dependency", "body"]);
});

it("initializes named expression and catch bindings in their own scopes", () => {
	const recursive = function require(n) {
		return n ? require(n - 1) : "named function";
	};
	const Named = class require {
		static read() { return require; }
	};
	expect(recursive(2)).toBe("named function");
	expect(Named.read()).toBe(Named);
	try {
		throw { require: () => "catch" };
	} catch ({ require }) {
		expect(require("./missing-catch")).toBe("catch");
	}
	expect(require("./value")).toBe("dependency");
});

it("runs declaration hooks for preinitialized runtime-name bindings", () => {
	function parameter(__webpack_require__) {
		return __webpack_require__("./missing-runtime-parameter");
	}
	const { __webpack_require__ } = { __webpack_require__: () => "local runtime" };
	expect(parameter(() => "parameter runtime")).toBe("parameter runtime");
	expect(__webpack_require__("./missing-runtime-local")).toBe("local runtime");
	expect(require("./value")).toBe("dependency");
});

it("overrides initialized AMD parameters without leaking their aliases", () => {
	const amd = require("./amd");
	expect(amd.value).toBe("dependency");
	expect(amd.local(() => "local")).toBe("local");
	expect(require("./amd-bound")).toBe("dependency");
	expect(require("./value")).toBe("dependency");
});

it("preserves AMD require callback handling", () => new Promise((resolve, reject) => {
	require(["require"], function (require) {
		var require;
		expect(require("./value")).toBe("dependency");
		resolve();
	}, error => reject(error));
}));

it("preserves require.ensure callback handling after initialization", () => new Promise((resolve, reject) => {
	require.ensure([], function (require) {
		var require;
		expect(require("./value")).toBe("dependency");
		resolve();
	}, error => reject(error));
}));

it("preserves arrow require.ensure callback handling", () => new Promise((resolve, reject) => {
	require.ensure([], require => {
		expect(require("./value")).toBe("dependency");
		resolve();
	}, error => reject(error));
}));

it("keeps import.then namespace tags on initialized parameters", () => {
	return import("./imports").then(namespace => {
		expect(namespace.imported).toBe("dependency");
	});
});
