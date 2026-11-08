import * as constExports from "./const-export";
import * as constDefaultExports from "./const-default-export";
import * as cyclicExports from "./cycle-a";
import * as cyclicDefaultExports from "./cycle-default-a";
import * as literalDefaultExports from "./literal-default-export";
import * as functionExports from "./function-export";
import * as letExports from "./let-export";
import * as varExports from "./var-export";
import * as mutatedVarExports from "./mutated-var-export";
import * as defaultFnExports from "./default-fn-export";
import * as mutatedDefaultFnExports from "./mutated-default-fn-export";
import * as renameAssignExports from "./rename-assign-export";
import * as blockVarRenameExports from "./block-var-rename-export";
import * as evalAliasExports from "./eval-alias-export";
import * as eagerThenExports from "./deferred-then-export";
import * as unrelatedValueExports from "./unrelated-value-export";
// Keep a defer edge so deferred-then-export is recorded as a defer target.
import defer * as _deferredThenExports from "./deferred-then-export";
void _deferredThenExports;

it("should bind const exports as readonly values", () => {
	expectValueDescriptor(constExports, "literal", "literal");
	expectValueDescriptor(constExports, "renamed", "local");
	expectValueDescriptor(constExports, "destructured", "destructured");
	expectValueDescriptor(constExports, "arrayValue", "array");
});

it("should bind const default exports as readonly values", () => {
	expectValueDescriptor(literalDefaultExports, "default", "literal-default");
	expectValueDescriptor(constDefaultExports, "default", "const-default");
});

it("should bind unmutated function and var exports as values", () => {
	expectValueDescriptor(functionExports, "fn", functionExports.fn);
	expectValueDescriptor(varExports, "foo", 1);
	expectValueDescriptor(varExports, "bar", 2);
	expectValueDescriptor(mutatedVarExports, "setFoo", mutatedVarExports.setFoo);
	expectValueDescriptor(letExports, "setCounter", letExports.setCounter);
	expectValueDescriptor(defaultFnExports, "default", defaultFnExports.default);
	expectValueDescriptor(
		mutatedDefaultFnExports,
		"setFn",
		mutatedDefaultFnExports.setFn
	);
});

it("should keep mutated default exports as getters", () => {
	expectGetterDescriptor(mutatedDefaultFnExports, "default");
	expect(mutatedDefaultFnExports.default()).toBe("default-fn");
	mutatedDefaultFnExports.setFn(() => "mutated");
	expect(mutatedDefaultFnExports.default()).toBe("mutated");
});

it("should keep mutated exports as getters", () => {
	expectGetterDescriptor(letExports, "counter");
	expect(letExports.counter).toBe(0);
	letExports.setCounter(5);
	expect(letExports.counter).toBe(5);

	expectGetterDescriptor(mutatedVarExports, "foo");
	expect(mutatedVarExports.foo).toBe(1);
	mutatedVarExports.setFoo(2);
	expect(mutatedVarExports.foo).toBe(2);
});

it("should keep exports mutated via rename-assignment as getters", () => {
	expectGetterDescriptor(renameAssignExports, "current");
	expect(renameAssignExports.current).toBe(null);
	renameAssignExports.init();
	expect(renameAssignExports.current).toBe(URL);
});

it("should keep block-scoped var rename aliases mutated in closures as getters", () => {
	expectGetterDescriptor(blockVarRenameExports, "current");
	expect(blockVarRenameExports.current).toBe(URL);
	blockVarRenameExports.reset();
	expect(blockVarRenameExports.current).toBe(null);
});

it("should keep exports in modules with direct eval as getters", () => {
	expectGetterDescriptor(evalAliasExports, "current");
	expect(evalAliasExports.current).toBe(URL);
	evalAliasExports.reset();
	expect(evalAliasExports.current).toBe(null);
});

it("should keep const exports in circular modules as getters", () => {
	expectGetterDescriptor(cyclicExports, "cyclicConst");
	expect(cyclicExports.readFromCycle()).toBe("cyclic");
});

it("should keep const default exports in circular modules as getters", () => {
	expectGetterDescriptor(cyclicDefaultExports, "default");
	expect(cyclicDefaultExports.readFromDefaultCycle()).toBe("cyclic-default");
});

it("should keep defer-target exports as getters when concatenateModules is disabled", () => {
	expectGetterDescriptor(eagerThenExports, "then");
	expectGetterDescriptor(eagerThenExports, "literal");
	expect(eagerThenExports.then()).toBe("then");
	expect(eagerThenExports.literal).toBe("deferred-literal");
});

it("should still bind unrelated exports as values when deferImport is enabled", () => {
	expectValueDescriptor(unrelatedValueExports, "literal", "unrelated-literal");
});

function expectValueDescriptor(ns, key, value) {
	const descriptor = Object.getOwnPropertyDescriptor(ns, key);
	expect(descriptor).toEqual(
		expect.objectContaining({
			enumerable: true,
			writable: false,
			value
		})
	);
	expect(descriptor.get).toBe(undefined);
}

function expectGetterDescriptor(ns, key) {
	const descriptor = Object.getOwnPropertyDescriptor(ns, key);
	expect(descriptor).toEqual(
		expect.objectContaining({
			enumerable: true,
			get: expect.any(Function)
		})
	);
	expect(Object.prototype.hasOwnProperty.call(descriptor, "value")).toBe(false);
}
