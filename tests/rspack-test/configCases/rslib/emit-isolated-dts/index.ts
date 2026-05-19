export interface Foo {
  value: string;
}

export type { Foo as TypeOnlyFoo } from "./types/foo";
export type { Foo as AliasFoo } from "@/foo";
export type { MixedFoo as AliasMixedFoo } from "@/mixed";
import { mtsValue } from "./module.mts";
export { mtsValue };

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
  expect(mtsValue).toEqual({ module: "mts" });
  expect(dts).toContain("export interface Foo");
  expect(dts).toContain("export type { Foo as TypeOnlyFoo }");
  expect(dts).toContain("./types/foo");
  expect(dts).toContain("export type { Foo as AliasFoo }");
  expect(dts).toContain("@/foo");
  expect(dts).toContain("export type { MixedFoo as AliasMixedFoo }");
  expect(dts).toContain("@/mixed");
  expect(dts).toContain("export declare const foo: Foo;");

  const typeOnlyDts = fs.readFileSync(
    path.resolve(
      __dirname,
      "../../../../configCases/rslib/emit-isolated-dts/dist/types/types/foo.d.ts",
    ),
    "utf-8",
  );

  expect(typeOnlyDts).toContain("export type Foo");
  expect(typeOnlyDts).toContain("label: string");

  const aliasDts = fs.readFileSync(
    path.resolve(
      __dirname,
      "../../../../configCases/rslib/emit-isolated-dts/dist/types/alias/foo.d.ts",
    ),
    "utf-8",
  );

  expect(aliasDts).toContain("export type Foo");
  expect(aliasDts).toContain("alias: boolean");

  const aliasMixedDts = fs.readFileSync(
    path.resolve(
      __dirname,
      "../../../../configCases/rslib/emit-isolated-dts/dist/types/alias/mixed.d.ts",
    ),
    "utf-8",
  );

  expect(aliasMixedDts).toContain("export type MixedFoo");
  expect(aliasMixedDts).toContain("mixed: string");
  expect(aliasMixedDts).toContain("export declare const mixedValue");

  const mtsDts = fs.readFileSync(
    path.resolve(
      __dirname,
      "../../../../configCases/rslib/emit-isolated-dts/dist/types/module.d.mts",
    ),
    "utf-8",
  );

  expect(mtsDts).toContain("export type MtsFoo");
  expect(mtsDts).toContain('module: "mts"');
  expect(mtsDts).toContain("export declare const mtsValue");
});
