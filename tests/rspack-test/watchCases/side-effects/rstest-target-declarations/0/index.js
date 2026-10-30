import { read } from "./importer.js";

it("refreshes target declarations without editing the importer", () => {
  expect(read()).toBe(42);
});
