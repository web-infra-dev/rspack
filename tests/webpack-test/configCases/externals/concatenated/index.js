import externalValue, { named as externalValueNamed } from "externalValue";

it("should ESM import a external value", function () {
	expect(externalValue).toBe("abc");
	expect(externalValueNamed).toBe(undefined);
});

import externalObject, { named as externalObjectNamed } from "externalObject";

it("should ESM import a external value", function () {
	expect(externalObject).toMatchObject({
		named: "named",
		default: "default"
	});
	expect(externalObjectNamed).toBe("named");
});

import externalEsModule, {
	named as externalEsModuleNamed
} from "externalEsModule";

it("should ESM import a external value", function () {
	expect(externalEsModule).toBe("default");
	expect(externalEsModuleNamed).toBe("named");
});
