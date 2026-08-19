import { restoreExternalPrototypeSetter } from "./external-prototype-setup";
import flaggedDefault, { value, getValue } from "./flagged";
import { a, b, inc } from "./plain";
import * as plainNs from "./plain";
import { setValue, getValue as getLiveValue } from "./live";
import { deep } from "./nested";
import {
	"a-b" as ab,
	a_b_612d62,
	chainedA,
	chainedB,
	"__rspack_cjs_external_setter__" as externalSetterExport,
	externalSetterObserved
} from "./weird-name";
import {
	anonymousFunction,
	anonymousArrow,
	AnonymousClass
} from "./anonymous-name";
import {
	defined,
	definedAnonymous,
	readDefined
} from "./define-name-collision";
import { argumentsType } from "./esm-arguments";
import {
	value as collisionValue,
	local as collisionLocal,
	placeholder as collisionPlaceholder,
	readGlobal as collisionReadGlobal
} from "./name-collision";

const externalSetterExportAtEvaluation = externalSetterExport;
const externalSetterObservedAtEvaluation = externalSetterObserved;
restoreExternalPrototypeSetter();

it("should provide named and default exports of a __esModule-flagged module", () => {
	expect(flaggedDefault).toBe("DEFAULT");
	expect(value).toBe(42);
	expect(getValue()).toBe(42);
});

it("should provide exports of a plain CommonJS module", () => {
	expect(a).toBe(1);
	expect(b).toBe(2);
	expect(inc()).toBe(1);
	expect(inc()).toBe(2);
});

it("should build a namespace object for whole-namespace usage", () => {
	expect(plainNs.a).toBe(1);
	expect(plainNs.b).toBe(2);
});

it("should keep live bindings for delayed export assignments", () => {
	expect(getLiveValue()).toBe(undefined);
	setValue(7);
	expect(getLiveValue()).toBe(7);
});

it("should support nested export assignments", () => {
	expect(deep.x).toBe("deep-x");
});

it("should support non-identifier export names", () => {
	expect(ab).toBe("a-b-value");
	expect(a_b_612d62).toBe("identifier-value");
});

it("should preserve chained CommonJS export assignments", () => {
	expect(chainedA).toBe("chained-value");
	expect(chainedB).toBe("chained-value");
});

it("should preserve inherited setters installed by an earlier module", () => {
	expect(externalSetterExportAtEvaluation).toBe(45);
	expect(externalSetterObservedAtEvaluation).toBe(45);
});

it("should preserve anonymous function and class names", () => {
	expect(anonymousFunction.name).toBe("");
	expect(anonymousArrow.name).toBe("");
	expect(AnonymousClass.name).toBe("");
});

it("should avoid generated CommonJS export name collisions", () => {
	expect(collisionValue).toBe(1);
	expect(collisionLocal).toBe(99);
	expect(collisionPlaceholder).toBe(98);
	expect(collisionReadGlobal()).toBe(97);
});

it("should avoid identifiers injected by presentational dependencies", () => {
	expect(defined).toBe(1);
	expect(definedAnonymous.name).toBe("");
	expect(readDefined()).toBe(96);
});

it("should keep concatenating ECMAScript modules that reference arguments", () => {
	expect(argumentsType).toBe("object");
});

it("should concatenate all CommonJS modules into the entry", () => {
	const concatModules = __STATS__.modules.filter((m) => m.modules);
	expect(concatModules.length).toBe(1);
	// index.js + external-prototype-setup.js + flagged.js + plain.js + live.js + nested.js + weird-name.js + name-collision.js + anonymous-name.js + define-name-collision.js + esm-arguments.js
	expect(concatModules[0].modules.length).toBe(11);
});
