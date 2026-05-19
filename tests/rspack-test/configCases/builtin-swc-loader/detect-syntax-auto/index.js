import js from "./value.js";
import jsx from "./value.jsx";
import mjs from "./value.mjs";
import ts from "./value.ts";
import tsx from "./value.tsx";
import mts from "./value.mts";
import cjs from "./value.cjs";
import cts from "./value.cts";

it("should detect parser syntax from resource extension", () => {
	expect(js).toBe("js");
	expect(jsx).toBe("jsx");
	expect(mjs).toBe("mjs");
	expect(cjs).toBe("cjs");
	expect(ts).toBe("ts");
	expect(tsx).toBe("tsx");
	expect(mts).toBe("mts");
	expect(cts).toBe("cts");
});
