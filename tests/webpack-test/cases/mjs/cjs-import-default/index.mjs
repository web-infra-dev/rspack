import * as star from "./cjs.js";
import def, { data } from "./cjs.js";
import jsonDef, * as jsonStar from "./data.json";
import * as dynamicStar from "./dynamic.js";
import dynamicDef, { data as dynamicData } from "./dynamic.js";
import * as dynamicFlaggedStar from "./dynamicFlagged.js";
import dynamicFlaggedDef, { data as dynamicFlaggedData } from "./dynamicFlagged.js";
import * as flaggedStar from "./flagged.js";
import flaggedDef, { data as flaggedData } from "./flagged.js";
import * as reexport from "./reexport.mjs";
import {
	data as data2, 
	default as def1,
	def as def2,
	ns
} from "./reexport.mjs";

it("should get correct values when importing named exports from a CommonJs module from mjs", function () {
	expect(typeof data).toBe("string");
	expect({ data }).toEqual({ data: "ok" });
	expect(def).toEqual({
		data: "ok",
		default: "default"
	});
	expect({ def }).toEqual({
		def: {
			data: "ok",
			default: "default"
		}
	});
	expect(star).toEqual(
		nsObj({
			default: {
				data: "ok",
				default: "default"
			},
			data: "ok"
		})
	);
	expect({ star }).toEqual({
		star: nsObj({
			default: {
				data: "ok",
				default: "default"
			},
			data: "ok"
		})
	});
	expect(star.default).toEqual({
		data: "ok",
		default: "default"
	});
	expect(ns).toEqual(
		nsObj({
			default: {
				data: "ok",
				default: "default"
			},
			data: "ok"
		})
	);
	expect(def1).toEqual({
		data: "ok",
		default: "default"
	});
	expect(def2).toEqual({
		data: "ok",
		default: "default"
	});
	expect(typeof data2).toBe("string");
	expect({ data2 }).toEqual({ data2: "ok" });
	expect(reexport).toEqual(
		nsObj({
			ns: nsObj({
				default: {
					data: "ok",
					default: "default"
				},
				data: "ok"
			}),
			default: {
				data: "ok",
				default: "default"
			},
			def: {
				data: "ok",
				default: "default"
			},
			data: "ok"
		})
	);
});

it("should get correct values when importing named exports from a flagged module from mjs", function () {
	expect(typeof flaggedData).toBe("string");
	expect({ flaggedData }).toEqual({ flaggedData: "ok" });
	expect(flaggedDef).toEqual({
		__esModule: true,
		data: "ok",
		default: "default"
	});
	expect({ flaggedDef }).toEqual({
		flaggedDef: {
			__esModule: true,
			data: "ok",
			default: "default"
		}
	});
	expect(flaggedStar).toEqual(
		nsObj({
			default: {
				__esModule: true,
				data: "ok",
				default: "default"
			},
			data: "ok"
		})
	);
	expect({ flaggedStar }).toEqual({
		flaggedStar: nsObj({
			default: {
				__esModule: true,
				data: "ok",
				default: "default"
			},
			data: "ok"
		})
	});
	expect(flaggedStar.default).toEqual({
		__esModule: true,
		data: "ok",
		default: "default"
	});
});

it("should get correct values when importing named exports from a dynamic (non-flagged) module from mjs", function () {
	expect(typeof dynamicData).toBe("string");
	expect({ dynamicData }).toEqual({ dynamicData: "ok" });
	expect(dynamicDef).toEqual({
		__esModule: false,
		data: "ok",
		default: "default"
	});
	expect({ dynamicDef }).toEqual({
		dynamicDef: {
			__esModule: false,
			data: "ok",
			default: "default"
		}
	});
	expect(dynamicStar).toEqual(
		nsObj({
			default: {
				__esModule: false,
				data: "ok",
				default: "default"
			},
			data: "ok"
		})
	);
	expect({ dynamicStar }).toEqual({
		dynamicStar: nsObj({
			default: {
				__esModule: false,
				data: "ok",
				default: "default"
			},
			data: "ok"
		})
	});
	expect(dynamicStar.default).toEqual({
		__esModule: false,
		data: "ok",
		default: "default"
	});
});

it("should get correct values when importing named exports from a dynamic (flagged) module from mjs", function () {
	expect(typeof dynamicFlaggedData).toBe("string");
	expect({ dynamicFlaggedData }).toEqual({ dynamicFlaggedData: "ok" });
	expect(dynamicFlaggedDef).toEqual({
		__esModule: true,
		data: "ok",
		default: "default"
	});
	expect({ dynamicFlaggedDef }).toEqual({
		dynamicFlaggedDef: {
			__esModule: true,
			data: "ok",
			default: "default"
		}
	});
	expect(dynamicFlaggedStar).toEqual(
		nsObj({
			default: {
				__esModule: true,
				data: "ok",
				default: "default"
			},
			data: "ok"
		})
	);
	expect({ dynamicFlaggedStar }).toEqual({
		dynamicFlaggedStar: nsObj({
			default: {
				__esModule: true,
				data: "ok",
				default: "default"
			},
			data: "ok"
		})
	});
	expect(dynamicFlaggedStar.default).toEqual({
		__esModule: true,
		data: "ok",
		default: "default"
	});
});

it("should get correct values when importing named exports from a default-only module from mjs", function () {
	expect(jsonDef).toEqual({
		__esModule: true,
		data: "ok",
		default: "default"
	});
	expect({ jsonDef }).toEqual({
		jsonDef: {
			__esModule: true,
			data: "ok",
			default: "default"
		}
	});
	expect(jsonStar).toEqual(
		nsObj({
			default: {
				__esModule: true,
				data: "ok",
				default: "default"
			}
		})
	);
	expect({ jsonStar }).toEqual({
		jsonStar: nsObj({
			default: {
				__esModule: true,
				data: "ok",
				default: "default"
			}
		})
	});
	expect(jsonStar.default).toEqual({
		__esModule: true,
		data: "ok",
		default: "default"
	});
});
