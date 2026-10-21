import fs from "node:fs";
import path from "node:path";
import normal from "./value.ts";
import url from "./value.ts?url";
import equal from "./data.txt?equal";
import defaultPriority from "./data.txt?default";

it("should skip SWC for a higher-priority URL rule", () => {
  expect(normal).toBe("normal");
  expect(url).toBe("url/value.ts");
  expect(fs.readFileSync(path.join(__dirname, url), "utf8")).toBe(
    'const value: string = "normal";\nexport default value;\n'
  );
});

it("should merge type and generator at equal priority and skip lower loaders", () => {
  expect(equal).toBe("equal/data.txt");
});

it("should merge omitted and explicit zero priorities", () => {
  expect(defaultPriority).toBe("priority fixture\n:zero");
});
