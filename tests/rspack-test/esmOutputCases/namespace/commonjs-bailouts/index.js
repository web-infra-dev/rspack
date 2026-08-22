import { before, after } from "./top-level-return.js";
import { toString as compoundAssignmentToString } from "./compound-assignment.js";
import {
	direct as directArguments,
	arrow as arrowArguments
} from "./factory-arguments.js";
import { type as typeofFactoryArguments } from "./typeof-factory-arguments.js";
import {
	read as readDeleteReference,
	remove as removeDeleteReference
} from "./delete-reference.js";
import {
	assign as assignDestructuring,
	value as destructuringValue
} from "./destructuring-assignment.js";
import {
	assign as assignForIn,
	value as forInValue
} from "./for-in-assignment.js";
import {
	assign as assignForOf,
	value as forOfValue
} from "./for-of-assignment.js";
import { run as runTaggedTemplate } from "./tagged-template.js";
import { value as prototypeRead } from "./prototype-read.js";
import { value as prototypeCustomRead } from "./prototype-custom-read.js";
import { toString as prototypeConditionalAssignmentToString } from "./prototype-conditional-assignment.js";
import {
	toString as prototypeCustomSetterExport,
	value as prototypeCustomSetterValue
} from "./prototype-custom-setter.js";
import {
	valueOf as prototypeIndirectObjectSetterExport,
	value as prototypeIndirectObjectSetterValue
} from "./prototype-indirect-object-setter.js";
import {
	toLocaleString as prototypeIndirectReflectSetterExport,
	value as prototypeIndirectReflectSetterValue
} from "./prototype-indirect-reflect-setter.js";
import { value as prototypeSetter } from "./prototype-setter.js";
import { value as prototypeUnknownRead } from "./prototype-unknown-read.js";
import mutableDefault, { value as mutableDefaultValue } from "./mutable-default.js";
import {
	increment as incrementUpdate,
	value as updateValue
} from "./update-assignment.js";

it("should keep CommonJS factory and exports object semantics", () => {
	expect(before).toBe("before");
	expect(after).toBeUndefined();
	expect(typeof compoundAssignmentToString).toBe("function");
	expect(directArguments).toBe(3);
	expect(arrowArguments).toBe(3);
	expect(typeofFactoryArguments).toBe("object");
	expect(readDeleteReference()).toBe(1);
	expect(removeDeleteReference()).toBe(true);
	expect(readDeleteReference()).toBeUndefined();
	expect(updateValue).toBe(1);
	expect(incrementUpdate()).toBe(1);
	expect(updateValue).toBe(2);
	expect(destructuringValue).toBe(1);
	assignDestructuring();
	expect(destructuringValue).toBe(2);
	expect(forInValue).toBe("");
	assignForIn();
	expect(forInValue).toBe("key");
	expect(forOfValue).toBe(1);
	assignForOf();
	expect(forOfValue).toBe(2);
	expect(runTaggedTemplate()).toBe(1);
	expect(prototypeRead).toBe("function");
	expect(prototypeCustomRead).toBe(42);
	expect(typeof prototypeConditionalAssignmentToString).toBe("function");
	expect(typeof prototypeCustomSetterExport).toBe("function");
	expect(prototypeCustomSetterValue).toBe(42);
	expect(typeof prototypeIndirectObjectSetterExport).toBe("function");
	expect(prototypeIndirectObjectSetterValue).toBe(42);
	expect(typeof prototypeIndirectReflectSetterExport).toBe("function");
	expect(prototypeIndirectReflectSetterValue).toBe(42);
	expect(prototypeSetter).toBe(42);
	expect(prototypeUnknownRead).toBeUndefined();
	expect(mutableDefault.value).toBe(1);
	expect(mutableDefaultValue).toBe(1);
	const mutate = (object) => {
		object.value = 2;
	};
	mutate(mutableDefault);
	expect(mutableDefault.value).toBe(2);
	expect(mutableDefaultValue).toBe(2);
});
