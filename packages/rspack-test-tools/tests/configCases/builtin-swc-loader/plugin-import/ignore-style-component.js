import { Button, FooBar } from "./src/ignore-style-component";

it("ignore-style-component", () => {
	expect(Button).toBe("Button");
	expect(FooBar).toBe("FooBar");
});
