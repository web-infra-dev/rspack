import { before, after } from "./top-level-return.js";
import {
	direct as directArguments,
	arrow as arrowArguments
} from "./factory-arguments.js";
import {
	read as readDeleteReference,
	remove as removeDeleteReference
} from "./delete-reference.js";
import { run as runTaggedTemplate } from "./tagged-template.js";
import { value as prototypeRead } from "./prototype-read.js";
import { value as prototypeSetter } from "./prototype-setter.js";
import mutableDefault, { value as mutableDefaultValue } from "./mutable-default.js";

it("should keep CommonJS factory and exports object semantics", () => {
	expect(before).toBe("before");
	expect(after).toBeUndefined();
	expect(directArguments).toBe(3);
	expect(arrowArguments).toBe(3);
	expect(readDeleteReference()).toBe(1);
	expect(removeDeleteReference()).toBe(true);
	expect(readDeleteReference()).toBeUndefined();
	expect(runTaggedTemplate()).toBe(1);
	expect(prototypeRead).toBe("function");
	expect(prototypeSetter).toBe(42);
	expect(mutableDefault.value).toBe(1);
	expect(mutableDefaultValue).toBe(1);
	const mutate = (object) => {
		object.value = 2;
	};
	mutate(mutableDefault);
	expect(mutableDefault.value).toBe(2);
	expect(mutableDefaultValue).toBe(2);
});
