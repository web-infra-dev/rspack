import * as umdStar from "./umd.js";
import umdDef, { umdData } from "./umd.js";

it("should get correct values when importing named exports from a umd module from mjs", function () {
	expect(umdDef).toEqual({
		data: "ok",
		default: "default"
	});
	expect({ umdDef }).toEqual({
		umdDef: {
			data: "ok",
			default: "default"
		}
	});
	expect(umdStar).toEqual(
		nsObj({
			default: {
				data: "ok",
				default: "default"
			},
			data: "ok"
		})
	);
	expect({ umdStar }).toEqual({
		umdStar: nsObj({
			default: {
				data: "ok",
				default: "default"
			},
			data: "ok"
		})
	});
	expect(umdStar.default).toEqual({
		data: "ok",
		default: "default"
	});
});
