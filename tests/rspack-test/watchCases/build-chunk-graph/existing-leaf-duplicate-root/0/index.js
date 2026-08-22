import "./child";

it("should preserve a root shared by named and global entries", () => {
  expect(globalThis.__duplicate_entry_root__).toBe(true);
});
