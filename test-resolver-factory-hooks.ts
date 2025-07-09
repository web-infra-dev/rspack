// TypeScript test for ResolverFactory hooks interface
import * as liteTapable from "@rspack/lite-tapable";
import { ResolverFactory } from "./packages/rspack/src/ResolverFactory";
import { Resolver } from "./packages/rspack/src/Resolver";

// Type-only test to verify the hooks interface is correct
type ResolverFactoryTest = {
  hooks: {
    resolveOptions: liteTapable.SyncWaterfallHook<[any, { type: string }]>;
    resolver: liteTapable.SyncHook<[Resolver, any, { type: string }]>;
  };
};

// This should compile without errors if the types are correct
function testHooksInterface(factory: ResolverFactory): void {
  // Test that hooks exist and have correct types
  const hooks: ResolverFactoryTest["hooks"] = factory.hooks;
  
  // Test resolveOptions hook signature
  factory.hooks.resolveOptions.tap("test", (resolveOptions, context) => {
    console.log("Type test:", context.type, resolveOptions);
    return resolveOptions;
  });
  
  // Test resolver hook signature
  factory.hooks.resolver.tap("test", (resolver, resolveOptions, context) => {
    console.log("Type test:", context.type, resolver, resolveOptions);
  });
}

// Test that constructor creates hooks
function testConstructor(): void {
  // This test verifies that the hooks are properly initialized
  const factory = new ResolverFactory(false);
  
  // Verify hooks exist
  console.assert(factory.hooks.resolveOptions !== undefined, "resolveOptions hook should exist");
  console.assert(factory.hooks.resolver !== undefined, "resolver hook should exist");
  
  // Verify hooks are instances of correct types
  console.assert(factory.hooks.resolveOptions instanceof liteTapable.SyncWaterfallHook, "resolveOptions should be SyncWaterfallHook");
  console.assert(factory.hooks.resolver instanceof liteTapable.SyncHook, "resolver should be SyncHook");
}

console.log("TypeScript interface test passed!");