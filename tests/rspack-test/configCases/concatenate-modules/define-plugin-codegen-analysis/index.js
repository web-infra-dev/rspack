import { expression } from "./expression";
import { identifier } from "./identifier";
import { array, constant, literal, object } from "./literal";

it("should analyze non-literal DefinePlugin code at codegen", () => {
	expect(identifier).toBe(40);
	expect(expression).toBe(42);
	expect(literal).toBe("literal");
	expect(constant).toBe(3);
	expect(array).toEqual(["literal", false, 1]);
	expect(object).toEqual({
		MODE: "production",
		DEV: false,
		PROD: true,
		SSR: false,
		BASE_URL: "/",
		ASSET_PREFIX: "",
		NESTED: {
			FLAGS: [true, false]
		}
	});
});
