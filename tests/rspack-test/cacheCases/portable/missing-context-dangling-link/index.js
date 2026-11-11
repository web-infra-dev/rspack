import fs from "node:fs";
import path from "node:path";
import kind from "./value";

it("should rebuild when a dangling link appears where a context directory was absent", async () => {
  if (COMPILER_INDEX === 0) {
    expect(kind).toBe("absent");
    fs.symlinkSync("never-created", path.join(CURRENT_CONTEXT, "config/optional-directory"));
    await NEXT_START(false);
  }
  if (COMPILER_INDEX === 1) {
    expect(kind).toBe("link to never-created");
    const stats = await NEXT_START(false);
    expect(stats.modules.filter(module => module.built)).toHaveLength(0);
  }
});
