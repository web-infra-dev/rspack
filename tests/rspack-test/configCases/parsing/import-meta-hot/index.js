import value from "./dep";

let typeofGuardMatched = false;

if (typeof import.meta.hot !== "undefined") {
	typeofGuardMatched = true;
}

if (import.meta.hot) {
	import.meta.hot.accept(["./dep"], () => {});
}

it("should support import.meta.hot as an HMR alias", () => {
	expect(value).toBe("import.meta.hot");
	expect(typeofGuardMatched).toBe(true);
	expect(typeof import.meta.hot).toBe("object");
	expect(typeof import.meta.hot.accept).toBe("function");
});
