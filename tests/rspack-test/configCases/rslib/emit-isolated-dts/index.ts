export interface Foo {
  value: string;
}

export const foo: Foo = { value: "bar" };

const fs = __non_webpack_require__("node:fs");
const path = __non_webpack_require__("node:path");

it("should emit declaration assets only through RslibPlugin", () => {
  const dts = fs.readFileSync(
    path.resolve(
      __dirname,
      "../../../../configCases/rslib/emit-isolated-dts/dist/types/index.d.ts",
    ),
    "utf-8",
  );

  expect(foo).toEqual({ value: "bar" });
  expect(dts).toContain("export interface Foo");
  expect(dts).toContain("export declare const foo: Foo;");
});
