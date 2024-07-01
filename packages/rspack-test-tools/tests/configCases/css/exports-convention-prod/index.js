import * as styles1 from "./style.module.css?camel-case#1";
import * as styles2 from "./style.module.css?camel-case#2";
import * as styles3 from "./style.module.css?camel-case#3";

const nsObjForWebTarget = m => {
	// some different between webpack experiments.css runtime
	// if (global.document) {
	// 	return nsObj(m);
	// }
	return m
}

it("should have correct value for css exports", () => {
	expect(styles1.classA).toBe("./style.module.css?camel-case#1-class-a");
	expect(styles1["class-b"]).toBe("./style.module.css?camel-case#1-class-b");
	expect(__webpack_require__("./style.module.css?camel-case#1")).toEqual(nsObjForWebTarget({
		"classA": "./style.module.css?camel-case#1-class-a",
    "class-b": "./style.module.css?camel-case#1-class-b",
	}))

	expect(styles2["class-a"]).toBe("./style.module.css?camel-case#2-class-a");
	expect(styles2.classA).toBe("./style.module.css?camel-case#2-class-a");
	expect(__webpack_require__("./style.module.css?camel-case#2")).toEqual(nsObjForWebTarget({
		"class-a": "./style.module.css?camel-case#2-class-a",
		"classA": "./style.module.css?camel-case#2-class-a",
	}))

	expect(styles3["class-a"]).toBe("./style.module.css?camel-case#3-class-a");
	expect(styles3.classA).toBe("./style.module.css?camel-case#3-class-a");
	expect(styles3["class-b"]).toBe("./style.module.css?camel-case#3-class-b");
	expect(styles3.classB).toBe("./style.module.css?camel-case#3-class-b");
	expect(__webpack_require__("./style.module.css?camel-case#3")).toEqual(nsObjForWebTarget({
		"class-a": "./style.module.css?camel-case#3-class-a",
		"classA": "./style.module.css?camel-case#3-class-a",
    "class-b": "./style.module.css?camel-case#3-class-b",
    "classB": "./style.module.css?camel-case#3-class-b",
	}))
});
