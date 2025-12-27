it('should be able to consume nested modules', async () => {
  const { default: main } = await import('package-1');
  expect(main('test')).toEqual('test package-1 package-2');
});