// Simple test to verify ResolverFactory hooks are working
const { ResolverFactory } = require('./packages/rspack/dist/index.js');

console.log('Testing ResolverFactory hooks...');

const resolverFactory = new ResolverFactory(false);

// Test that hooks exist
console.log('resolveOptions hook exists:', !!resolverFactory.hooks.resolveOptions);
console.log('resolver hook exists:', !!resolverFactory.hooks.resolver);

// Test resolveOptions hook
let resolveOptionsHookCalled = false;
resolverFactory.hooks.resolveOptions.tap('TestPlugin', (resolveOptions, context) => {
  console.log('resolveOptions hook called with:', { type: context.type, resolveOptions });
  resolveOptionsHookCalled = true;
  // Return modified options
  return { ...resolveOptions, test: true };
});

// Test resolver hook
let resolverHookCalled = false;
resolverFactory.hooks.resolver.tap('TestPlugin', (resolver, resolveOptions, context) => {
  console.log('resolver hook called with:', { type: context.type, resolverExists: !!resolver });
  resolverHookCalled = true;
});

// Create a resolver to trigger hooks
try {
  const resolver = resolverFactory.get('normal', { mainFields: ['main'] });
  console.log('Resolver created successfully:', !!resolver);
  
  console.log('resolveOptions hook was called:', resolveOptionsHookCalled);
  console.log('resolver hook was called:', resolverHookCalled);
  
  if (resolveOptionsHookCalled && resolverHookCalled) {
    console.log('✅ All tests passed! ResolverFactory hooks are working correctly.');
  } else {
    console.log('❌ Some hooks were not called.');
  }
} catch (error) {
  console.error('Error creating resolver:', error);
}