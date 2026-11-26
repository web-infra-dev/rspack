import {
	plainFunction,
	arrowFunction,
	functionExpression,
	nestedFunctionThis,
	nestedClassThis,
	renamedLater,
	usesThis,
	arrowThis,
	parameterThis,
	computedKeyThis,
	reassigned,
	reassign
} from "./functions";
import defaultFunction from "./default-function";
import defaultArrow from "./default-arrow";
import defaultIdentifier from "./default-identifier";
import defaultThis from "./default-this";
import { evalNeighbor, runEval } from "./eval-module";

it("should keep the same results for this-free exports", () => {
	expect(plainFunction()).toBe("plain");
	expect(arrowFunction()).toBeUndefined();
	expect(functionExpression()).toBe("expression");
	expect(nestedFunctionThis()()).toBeUndefined();
	expect(new (nestedClassThis())().method()).toBeInstanceOf(Object);
	expect(renamedLater()).toBe("later");
	expect(defaultFunction()).toBe("default function");
	expect(defaultArrow()).toBe("default arrow");
	expect(defaultIdentifier()).toBe("default identifier");
	expect(evalNeighbor()).toBe("neighbor");
	expect(runEval()).toBe(2);
});

it("should call exports that read this without a receiver", () => {
	expect(usesThis()).toBeUndefined();
	expect(arrowThis()).toBeUndefined();
	expect(parameterThis()).toBeUndefined();
	expect(Object.getOwnPropertyNames(computedKeyThis().prototype)).toContain(
		"undefined"
	);
	expect(defaultThis()).toBeUndefined();
	expect(reassigned()).toBe("original");
	reassign();
	expect(reassigned()).toBeUndefined();
});

