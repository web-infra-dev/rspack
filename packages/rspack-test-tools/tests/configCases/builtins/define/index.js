import DO_NOT_CONVERTED8, { DO_NOT_CONVERTED7 } from "./lib";

const lib = require("./lib");
const { DO_NOT_CONVERTED9 } = require("./lib");

it("should builtins define works", () => {
	expect(TRUE).toBe(true);
	expect(FALSE).toBe(false);
	expect(TRUE_STRING).toBe(true);
	expect(FALSE_STRING).toBe(false);
	expect(NUMBER_ADD).toBe(5);
	expect(NULL).toBe(null);
	expect(UNDEFINED).toBe(undefined);
	expect(UNDEFINED_STRING).toBe(undefined);

	expect(FUNCTION(5)).toBe(6);
	expect(typeof FUNCTION).toBe("function");
	expect(NUMBER).toBe(100.05);
	expect(ZERO).toBe(0);

	let ZERO_OBJ = { ZERO: 0 };
	expect(ZERO_OBJ.ZERO).toBe(0);
	expect(ZERO_OBJ[ZERO]).toBe(undefined);
	expect(ZERO_OBJ[0]).toBe(undefined);
	expect(ZERO_OBJ["ZERO"]).toBe(0);
	expect(BIGINT).toBe(10000n);
	expect(BIGINT2).toBe(100000000000n);
	expect(POSITIVE_ZERO).toBe(0);
	expect(NEGATIVE_ZERO).toBe(-0);
	expect(POSITIVE_NUMBER).toBe(100.25);
	expect(NEGATIVE_NUMBER).toBe(-100.25);
	expect(STRING).toBe("string");
	expect(EMPTY_STRING).toBe("");
	expect("abc").toMatch(REGEXP);
	// expect(ZERO.ABC).toBe(undefined);

	let error_count = 0;
	try {
		error_count += 1;
		MEMBER_PROPS_SHOULD_DO_NOT_CONVERTED.ZERO;
		error_count += 1;
	} catch (err) {}
	expect(error_count).toBe(1);

	try {
		error_count += 1;
		MEMBER_PROPS_SHOULD_DO_NOT_CONVERTED.REGEXP.REGEXP;
		error_count += 1;
	} catch (err) {}
	expect(error_count).toBe(2);

	expect(ARRAY).toEqual([300, ["six"]]);
	expect(ARRAY[0]).toBe(300);
	expect(ARRAY[0][1]).toBe(undefined);
	expect(ARRAY[1]).toEqual(["six"]);
	expect(ARRAY[1][0]).toBe("six");
	expect(ARRAY[1][0][0]).toBe("s");
	expect(ARRAY[ONE]).toEqual(["six"]);
	expect(ARRAY[ARRAY]).toBe(undefined);

	expect(OBJECT.OBJ).toEqual({ NUM: 1 });
	expect(OBJECT.OBJ.NUM).toBe(1);
	expect(OBJECT.UNDEFINED).toBe(undefined);
	expect("def").toMatch(OBJECT.REGEXP);
	expect(OBJECT.STR).toBe("string");
	expect(OBJECT.AAA).toBe(undefined);

	expect(P1.P2.P3).toBe(301);
	expect(P1.P2.P4).toBe("302");
	expect(P1).toBe(303);
	expect(P1.P2).toBe(304);

	// expect(P1.P4).toBe(undefined); // "303.P4"

	try {
		error_count += 1;
		P4.P1;
		error_count += 1;
	} catch (err) {}
	expect(error_count).toBe(3);

	expect(P1.P2.P4.P1).toBe(undefined);
	expect(P1.P2.P4.P3).toBe(undefined);
	expect(P1.P2.P4.P4).toBe(undefined);

	const DO_NOT_CONVERTED = 201;
	expect(DO_NOT_CONVERTED).toBe(201);
	let { DO_NOT_CONVERTED2 } = { DO_NOT_CONVERTED2: 202 };
	expect(DO_NOT_CONVERTED2).toBe(202);
	const { c: DO_NOT_CONVERTED3 } = { c: 203 };
	expect(DO_NOT_CONVERTED3).toBe(203);
	try {
		error_count += 1;
		DO_NOT_CONVERTED4;
		error_count += 1;
	} catch (err) {}
	expect(error_count).toBe(4);
	let DO_NOT_CONVERTED4 = 204;

	const USELESS = {
		ZERO: 0
	};
	{
		const A = DO_NOT_CONVERTED4;
		expect(A).toBe(204);

		const DO_NOT_CONVERTED3 = 205;
		expect(DO_NOT_CONVERTED3).toBe(205);

		const B = ZERO;
		expect(B).toBe(0);

		let IN_BLOCK = 2;
		expect(IN_BLOCK).toBe(2);

		{
			{
				{
					expect(SHOULD_CONVERTED).toBe(205);
				}
			}
		}
	}

	try {
		error_count += 1;
		IN_BLOCK;
		error_count += 1;
	} catch (err) {}
	expect(error_count).toBe(5);

	expect(USELESS).toEqual({ ZERO: 0 });
	expect({}.DO_NOT_CONVERTED5).toBe(undefined);
	expect(M1.M2.M3.DO_NOT_CONVERTED6).toBe(undefined);
	expect(DO_NOT_CONVERTED7).toBe(402);
	expect(DO_NOT_CONVERTED8).toBe(401);
	expect(DO_NOT_CONVERTED9).toBe(403);
	expect(lib.DO_NOT_CONVERTED9).toBe(403);

	try {
		error_count += 1;
		M1;
		error_count += 1;
	} catch (err) {}
	expect(error_count).toBe(6);

	try {
		error_count += 1;
		aa = SHOULD_CONVERTED;
		error_count += 1;
	} catch (err) {}
	expect(error_count).toBe(7);

	expect(SHOULD_CONVERTED == 205).toBe(true);
	expect(207 == SHOULD_CONVERTED).toBe(false);

	try {
		error_count += 1;
		CONVERTED_TO_MEMBER;
		error_count += 1;
	} catch (err) {}
	expect(error_count).toBe(8);

	// TODO: recursive
	// assert.equal(wurst).toBe(unde);
	// assert.equal(suppe).toBe(wurst);
});
