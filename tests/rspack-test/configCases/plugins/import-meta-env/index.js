function loadEnv() {
	return { PRESENT: "present" };
}

it("should expose NODE_ENV from mode (WebpackOptionsApply)", () => {
	const env = import.meta.env;
	expect(env.NODE_ENV).toBe("production");
});

it("should expose variables from EnvironmentPlugin", () => {
	const env = import.meta.env;
	expect(env.ENV_VAR_FROM_ENV).toBe("from_environment_plugin");
});

it("should expose variables from DefinePlugin", () => {
	const env = import.meta.env;
	expect(env.CUSTOM_VAR).toBe("custom_value");
});

it("should keep direct access and object replacement consistent for duplicate definitions", () => {
	const env = import.meta.env;
	expect(import.meta.env.ORDERED_VAR).toBe("first_define_plugin");
	expect(env.ORDERED_VAR).toBe(import.meta.env.ORDERED_VAR);
});

it("should not mirror import.meta.env definitions to process.env", () => {
	const env = import.meta.env;
	expect(env.ONLY_IMPORT_META).toBe("only_import_meta");
	expect(process.env.ONLY_IMPORT_META).not.toBe("only_import_meta");
});

it("should not collect user process.env definitions into import.meta.env", () => {
	const env = import.meta.env;
	expect(process.env.PROCESS_ONLY).toBe("process_only");
	expect(env.PROCESS_ONLY).toBe(undefined);
});

it("should emit __proto__ env keys as own properties", () => {
	const env = import.meta.env;
	expect(Object.prototype.hasOwnProperty.call(env, "__proto__")).toBe(true);
	expect(env.__proto__).toBe("proto_value");
});

it("should support typeof import.meta.env", () => {
	expect(typeof import.meta.env).toBe("object");
});

it("should evaluate typeof import.meta.env as 'object'", () => {
	const typeofEnv = typeof import.meta.env;
	expect(typeofEnv).toBe("object");
});

it("should treat import.meta.env as truthy", () => {
	if (import.meta.env) {
		expect(true).toBe(true);
	} else {
		throw new Error("import.meta.env should be truthy");
	}
});

it("should wrap import.meta.env object literals in expression context", () => {
	import.meta.env;
	expect(import.meta.env && true).toBe(true);
});

it("should treat import.meta.env.NOT_EXIST as falsy", () => {
	if (import.meta.env.NOT_EXIST) {
		throw new Error("import.meta.env should be falsy");
	} else {
		expect(true).toBe(true);
	}
});

it("should treat import.meta.env.NOT_EXIST as falsy", () => {
	const NOT_EXIST = import.meta.env.NOT_EXIST;
	if (NOT_EXIST) {
		throw new Error("import.meta.env should be falsy");
	} else {
		expect(true).toBe(true);
	}
});

it("should evaluate typeof for an undefined env field", () => {
	expect(typeof import.meta.env.TYPEOF_UNKNOWN).toBe("undefined");
});

it("should support typeof-only import.meta.env definitions", () => {
	expect(typeof import.meta.env.TYPEOF_DEFINED).toBe("string");
	expect(import.meta.env.TYPEOF_DEFINED).toBe(undefined);
});

it("should preserve an env field explicitly defined as undefined", () => {
	expect(import.meta.env.EXPLICIT_UNDEFINED).toBe(undefined);
});

it("should preserve whole-object import.meta.env definitions", () => {
	const env = import.meta.env;
	expect(env.MODE).toBe("production");
	expect(env.FEATURE).toBe("enabled");
	expect(env.PER_KEY).toBe("per-key");
	expect(env.FROM_ENVIRONMENT_PLUGIN).toBe("from-environment-plugin");
	expect(env.NESTED.FROM_OBJECT).toBe("nested-object");
	expect(env.NESTED.PER_KEY).toBe("nested-per-key");
	expect(env.OBJECT_FORM.VALUE).toBe("object-form");
	expect(env["DOT.KEY"]).toBe("dot-key");
	expect(Array.isArray(env.OBJECT_FORM.ARRAY)).toBe(true);
	expect(env.OBJECT_FORM.ARRAY).toEqual(["a", "b"]);
	expect(env.OBJECT_FORM["DOT.KEY"]).toBe("object-dot");
	expect(env.OBJECT_FORM.DOT).toBeUndefined();
	expect(import.meta.env.OBJECT_FORM.ARRAY).toEqual(["a", "b"]);
	expect(import.meta.env.OBJECT_FORM["DOT.KEY"]).toBe("object-dot");
	const { OBJECT_FORM: { ARRAY: [first, second] } } = import.meta.env;
	expect([first, second]).toEqual(["a", "b"]);
	expect(Object.prototype.hasOwnProperty.call(env, "NESTED.PER_KEY")).toBe(
		false
	);
	expect(Object.prototype.hasOwnProperty.call(env, "OBJECT_FORM.VALUE")).toBe(
		false
	);
	expect(import.meta.env.MODE).toBe("production");
	expect(import.meta.env.PER_KEY).toBe("per-key");
	expect(import.meta.env.NESTED.PER_KEY).toBe("nested-per-key");
	expect(import.meta.env["DOT.KEY"]).toBe("dot-key");
	expect(() => import.meta.env.DOT.KEY).toThrow(TypeError);
	expect(import.meta.env.FROM_ENVIRONMENT_PLUGIN).toBe(
		"from-environment-plugin"
	);
	const { MODE, FEATURE, PER_KEY, FROM_ENVIRONMENT_PLUGIN } = import.meta.env;
	expect({ MODE, FEATURE, PER_KEY, FROM_ENVIRONMENT_PLUGIN }).toEqual({
		MODE: "production",
		FEATURE: "enabled",
		PER_KEY: "per-key",
		FROM_ENVIRONMENT_PLUGIN: "from-environment-plugin"
	});
	expect(env.UNKNOWN).toBe(undefined);
	expect(import.meta.env.UNKNOWN).toBe(undefined);
});

it("should evaluate an undefined nested env field", () => {
	expect(import.meta.env.NESTED.MISSING).toBe(undefined);
});

it("should evaluate an undefined destructured env field", () => {
	const { DESTRUCTURED_UNKNOWN } = import.meta.env;
	expect(DESTRUCTURED_UNKNOWN).toBe(undefined);
});

it("should support nested destructuring under a dynamic env value", () => {
	const {
		DYNAMIC: { PRESENT }
	} = import.meta.env;
	expect(PRESENT).toBe("present");
});

it("should statically filter top-level env destructuring", () => {
	delete globalThis.__IMPORT_META_ENV_UNUSED__;
	const { DESTRUCTURED_USED } = import.meta.env;
	expect(DESTRUCTURED_USED).toBe("destructured-used");
	expect(globalThis.__IMPORT_META_ENV_UNUSED__).toBeUndefined();
});

it("should statically filter env destructuring from import.meta", () => {
	delete globalThis.__IMPORT_META_ENV_UNUSED__;
	const {
		env: { DESTRUCTURED_USED }
	} = import.meta;
	expect(DESTRUCTURED_USED).toBe("destructured-used");
	expect(globalThis.__IMPORT_META_ENV_UNUSED__).toBeUndefined();
});

it("should render nested env objects completely", () => {
	delete globalThis.__IMPORT_META_ENV_NESTED_UNUSED__;
	const {
		NESTED_DESTRUCTURING: { USED }
	} = import.meta.env;
	expect(USED).toBe("nested-used");
	expect(globalThis.__IMPORT_META_ENV_NESTED_UNUSED__).toBe(true);
	delete globalThis.__IMPORT_META_ENV_NESTED_UNUSED__;
});

it("should render all env fields for rest destructuring", () => {
	delete globalThis.__IMPORT_META_ENV_UNUSED__;
	const { DESTRUCTURED_USED, ...rest } = import.meta.env;
	expect(DESTRUCTURED_USED).toBe("destructured-used");
	expect(rest.DESTRUCTURED_UNUSED).toBe(true);
	expect(globalThis.__IMPORT_META_ENV_UNUSED__).toBe(true);
	delete globalThis.__IMPORT_META_ENV_UNUSED__;
});


it("should preserve member access after an unknown env field", () => {
	const env = import.meta.env;
	expect(() => env.UNKNOWN_CHAIN.value).toThrow(TypeError);
	expect(() => import.meta.env.UNKNOWN_CHAIN.value).toThrow(TypeError);
});
