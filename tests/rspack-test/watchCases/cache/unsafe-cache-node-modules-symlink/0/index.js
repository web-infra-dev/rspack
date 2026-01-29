import { version } from "lib";

it("should detect changes in symlinked node_modules when using persistent cache", () => {
    if (WATCH_STEP === "0") {
        expect(version).toBe("v1");
    } else {
        expect(version).toBe("v2");
    }
});
