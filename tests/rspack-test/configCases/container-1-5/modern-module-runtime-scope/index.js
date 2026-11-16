it('reads the MF instance initialized by the ESM runtime', () => {
  expect(__rspack_context.federation.instance.options.name).toBe('scope_host');
});
