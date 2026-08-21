import { restoreExternalPrototypeSetter } from "./external-prototype-setup.js";
import {
	"a-b" as ab,
	"__rspack_cjs_external_setter__" as externalSetterExport,
	getValue,
	anonymousFunction,
	anonymousArrow,
	AnonymousClass,
	defined,
	definedAnonymous,
	escapedDefined,
	local,
	placeholder,
	readGlobal,
	nestedValue,
	readNestedValue,
	setNestedValue,
	readDefined,
	readEscapedDefined,
	parenthesizedLeft,
	parenthesizedRight,
	parenthesizedBoth,
	parenthesizedSequence,
	setValue,
	value,
	a_b_612d62,
	chainedA,
	chainedB,
	externalGetterReadsDuringEvaluation,
	externalSetterObserved
} from "./foo.js";

const externalSetterExportAtEvaluation = externalSetterExport;
const externalGetterReadsDuringEvaluationAtEvaluation =
	externalGetterReadsDuringEvaluation;
const externalSetterObservedAtEvaluation = externalSetterObserved;
restoreExternalPrototypeSetter();

it("should scope-hoist a statically analyzable CommonJS module", () => {
	expect(value).toBe(1);
	expect(getValue()).toBe(1);
	expect(local).toBe(41);
	expect(placeholder).toBe(42);
	expect(readGlobal()).toBe(43);
	expect(nestedValue).toBe(1);
	expect(readNestedValue()).toBe(1);
	setNestedValue(2);
	expect(nestedValue).toBe(2);
	expect(readNestedValue()).toBe(2);
	expect(anonymousFunction.name).toBe("");
	expect(anonymousArrow.name).toBe("");
	expect(AnonymousClass.name).toBe("");
	expect(defined).toBe(1);
	expect(definedAnonymous.name).toBe("");
	expect(readDefined()).toBe(44);
	expect(escapedDefined).toBe(1);
	expect(readEscapedDefined()).toBe(46);
	expect(parenthesizedLeft).toBe(1);
	expect(parenthesizedRight).toBe(2);
	expect(parenthesizedBoth).toBe(3);
	expect(parenthesizedSequence).toBe(4);
	expect(ab).toBe("a-b-value");
	expect(a_b_612d62).toBe("identifier-value");
	expect(chainedA).toBe("chained-value");
	expect(chainedB).toBe("chained-value");
	expect(externalSetterExportAtEvaluation).toBe(45);
	expect(externalGetterReadsDuringEvaluationAtEvaluation).toBe(0);
	expect(externalSetterObservedAtEvaluation).toBe(45);

  setValue(2);

  expect(value).toBe(2);
  expect(getValue()).toBe(2);
});
