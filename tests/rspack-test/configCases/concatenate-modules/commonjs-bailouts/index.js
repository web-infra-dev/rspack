import * as callContext from "./call-context";
import * as callOnly from "./call-only";
import * as deleteReference from "./delete-reference";
import * as defineProperty from "./define-property";
import * as escaped from "./escaped";
import * as exportRequire from "./export-require";
import * as factoryArguments from "./factory-arguments";
import * as moduleId from "./module-id";
import * as prototypeRead from "./prototype-read";
import * as prototypeSetter from "./prototype-setter";
import * as reassign from "./reassign";
import * as sloppy from "./sloppy";
import * as taggedTemplate from "./tagged-template";
import * as thisExports from "./this-exports";
import * as thisRead from "./this-read";
import * as topLevelReturn from "./top-level-return";

it("should keep unsupported CommonJS modules working", () => {
	expect(sloppy.s).toBe("sloppy");
	expect(reassign.default.r).toBe("reassigned");
	expect(thisExports.t).toBe("this-export");
	expect(callContext.run()).toBe("ctx-ok");
	expect(callOnly.g()).toBe("f-g");
	expect(typeof moduleId.id).not.toBe("undefined");
	expect(defineProperty.d).toBe("defined");
	expect(escaped.e).toBe("escaped");
	expect(exportRequire.inner.s).toBe("sloppy");
	expect(thisRead.viaThis).toBe("v");
	expect(topLevelReturn.before).toBe("before");
	expect(topLevelReturn.after).toBeUndefined();
	expect(factoryArguments.direct).toBe(3);
	expect(factoryArguments.arrow).toBe(3);
	expect(deleteReference.read()).toBe(1);
	expect(deleteReference.remove()).toBe(true);
	expect(deleteReference.read()).toBeUndefined();
	expect(taggedTemplate.run()).toBe(1);
	expect(prototypeRead.value).toBe("function");
	expect(prototypeSetter.value).toBe(42);
});

it("should not concatenate any of the unsupported modules", () => {
	const concatModules = __STATS__.modules.filter((m) => m.modules);
	expect(concatModules.length).toBe(0);
});

it("should report a bailout reason for each unsupported module", () => {
	/**
	 * @param {string} name module name
	 * @returns {string[]} optimization bailout messages
	 */
	const bailoutsOf = (name) => {
		const module = __STATS__.modules.find((m) => m.name === `./${name}`);
		expect(module).toBeDefined();
		return module.optimizationBailout || [];
	};
	expect(bailoutsOf("sloppy.js")).toContainEqual(
		expect.stringContaining("not in strict mode")
	);
	expect(bailoutsOf("this-exports.js")).toContainEqual(
		expect.stringContaining("uses this to define exports")
	);
	expect(bailoutsOf("this-read.js")).toContainEqual(
		expect.stringContaining("references its exports via this")
	);
	expect(bailoutsOf("call-only.js")).toContainEqual(
		expect.stringContaining("call context")
	);
	expect(bailoutsOf("module-id.js")).toContainEqual(
		expect.stringContaining("module.id")
	);
	expect(bailoutsOf("define-property.js")).toContainEqual(
		expect.stringContaining("Object.defineProperty(exports)")
	);
	expect(bailoutsOf("export-require.js")).toContainEqual(
		expect.stringContaining("unsupported dependency")
	);
	expect(bailoutsOf("top-level-return.js")).toContainEqual(
		expect.stringContaining("top-level return")
	);
	expect(bailoutsOf("factory-arguments.js")).toContainEqual(
		expect.stringContaining("CommonJS arguments")
	);
	expect(bailoutsOf("delete-reference.js")).toContainEqual(
		expect.stringContaining("delete on CommonJS exports")
	);
	expect(bailoutsOf("tagged-template.js")).toContainEqual(
		expect.stringContaining("call context")
	);
	expect(bailoutsOf("prototype-read.js")).toContainEqual(
		expect.stringContaining("Object.prototype")
	);
	expect(bailoutsOf("prototype-setter.js")).toContainEqual(
		expect.stringContaining("assigns to exports.__proto__")
	);
});
