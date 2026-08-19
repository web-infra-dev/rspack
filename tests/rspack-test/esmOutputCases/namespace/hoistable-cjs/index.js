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
	local,
	placeholder,
	readGlobal,
	readDefined,
	setValue,
	value,
	a_b_612d62,
	chainedA,
	chainedB,
	externalSetterObserved
} from "./foo.js";

const externalSetterExportAtEvaluation = externalSetterExport;
const externalSetterObservedAtEvaluation = externalSetterObserved;
restoreExternalPrototypeSetter();

it("should scope-hoist a statically analyzable CommonJS module", () => {
	expect(value).toBe(1);
	expect(getValue()).toBe(1);
	expect(local).toBe(41);
	expect(placeholder).toBe(42);
	expect(readGlobal()).toBe(43);
	expect(anonymousFunction.name).toBe("");
	expect(anonymousArrow.name).toBe("");
	expect(AnonymousClass.name).toBe("");
	expect(defined).toBe(1);
	expect(definedAnonymous.name).toBe("");
	expect(readDefined()).toBe(44);
	expect(ab).toBe("a-b-value");
	expect(a_b_612d62).toBe("identifier-value");
	expect(chainedA).toBe("chained-value");
	expect(chainedB).toBe("chained-value");
	expect(externalSetterExportAtEvaluation).toBe(45);
	expect(externalSetterObservedAtEvaluation).toBe(45);

  setValue(2);

  expect(value).toBe(2);
  expect(getValue()).toBe(2);
});
