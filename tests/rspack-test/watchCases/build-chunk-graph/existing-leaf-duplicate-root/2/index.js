import "./child";

it("should preserve a root shared by named and global entries", () => {
  expect(globalThis.__duplicate_entry_root__).toBe(true);
});

it("should rebuild when the global entry changes to an existing leaf", () => {
  expect(globalThis.__changed_global_entry_root__).toBe(true);
});
