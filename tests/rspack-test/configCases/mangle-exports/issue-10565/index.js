import { obj3, default as a } from "./non-exists" // enable side effects to ensure reexport is not skipped

it("should not panic", () => {
	const { aaa, bbb } = obj3;
	const { ccc, ddd } = a;
	aaa, bbb;
	ccc, ddd;
});

