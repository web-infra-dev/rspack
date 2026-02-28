import { Button, FooBar } from "./src/ignore-es-component";

it("ignore-es-component", () => {
	expect(Button).toBe("Button");
	expect(FooBar).toBe("FooBar");
});
