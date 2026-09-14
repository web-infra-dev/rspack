import { light as inheritedFalse } from "inherited-false";
import { light as inheritedGlob } from "inherited-glob";
import { light as explicitTrue } from "explicit-true";
import { light as namedBoundary } from "named-boundary";
import { light as anonymousBoundary } from "anonymous-boundary";
import { light as localNested } from "./local-nested/esm/index.js";

it("inherits sideEffects from the owning package past a type-only package.json", () => {
  expect(inheritedFalse).toBe("inherited-false");
  expect(globalThis.__nestedSideEffectsMarkers).not.toContain("inherited-false");
});

it("matches inherited sideEffects globs relative to the owning package root", () => {
  expect(inheritedGlob).toBe("inherited-glob");
  expect(globalThis.__nestedSideEffectsMarkers).not.toContain("inherited-glob");
});

it("honors an explicit sideEffects value on the nested package.json", () => {
  expect(explicitTrue).toBe("explicit-true");
  expect(globalThis.__nestedSideEffectsMarkers).toContain("explicit-true");
});

it("stops inheritance at named package and node_modules boundaries", () => {
  expect(namedBoundary).toBe("named-boundary");
  expect(anonymousBoundary).toBe("anonymous-boundary");
  expect(globalThis.__nestedSideEffectsMarkers).toContain("named-boundary");
  expect(globalThis.__nestedSideEffectsMarkers).toContain("anonymous-boundary");
});

it("keeps app-local nested package.json shadowing behavior", () => {
  expect(localNested).toBe("local-nested");
  expect(globalThis.__nestedSideEffectsMarkers).toContain("local-nested");
});
