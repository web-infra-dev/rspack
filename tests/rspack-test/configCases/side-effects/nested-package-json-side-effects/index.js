import { light as extra4 } from "@scope/inherited";
import { light as extra3 } from "explicit-empty";
import { light as extra2 } from "explicit-false";
import { light as extra1 } from "inherited-string";
import { light as extra0 } from "inherited-kept";
import { light as inheritedFalse } from "inherited-false";
import { light as inheritedGlob } from "inherited-glob";
import { light as explicitTrue } from "explicit-true";
import { light as namedBoundary } from "named-boundary";
import { light as anonymousBoundary } from "anonymous-boundary";
import { light as localNested } from "./local-nested/esm/index.js";
import { light as directoryMetadata } from "directory-metadata";

it("inherits sideEffects from the owning package past a type-only package.json", () => {
  expect(inheritedFalse).toBe("inherited-false");
  expect(globalThis.__nestedSideEffectsMarkers).not.toContain("inherited-false");
});

it("skips ancestor package.json directories when inheriting sideEffects", () => {
  expect(directoryMetadata).toBe("directory-metadata");
  expect(globalThis.__nestedSideEffectsMarkers).not.toContain("directory-metadata");
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

it("respects inherited-kept metadata", () => {
  expect(extra0).toBe("inherited-kept");
  expect(globalThis.__nestedSideEffectsMarkers).toContain("inherited-kept");
});

it("respects inherited-string metadata", () => {
  expect(extra1).toBe("inherited-string");
  expect(globalThis.__nestedSideEffectsMarkers).toContain("inherited-string");
});

it("respects explicit-false metadata", () => {
  expect(extra2).toBe("explicit-false");
  expect(globalThis.__nestedSideEffectsMarkers).not.toContain("explicit-false");
});

it("respects explicit-empty metadata", () => {
  expect(extra3).toBe("explicit-empty");
  expect(globalThis.__nestedSideEffectsMarkers).not.toContain("explicit-empty");
});

it("respects @scope/inherited metadata", () => {
  expect(extra4).toBe("@scope/inherited");
  expect(globalThis.__nestedSideEffectsMarkers).not.toContain("@scope/inherited");
});
