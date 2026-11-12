import remote from "remote/Module";
import shared from "shared";

it("should load static remotes and shared modules before async startup", () => {
	expect(remote).toBe("remote");
	expect(shared).toBe("shared");
});
