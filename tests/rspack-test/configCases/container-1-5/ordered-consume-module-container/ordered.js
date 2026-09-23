import lib from 'lib';

it('should install the initial ordered consume before its shared-runtime entry runs', () => {
  expect(lib).toBe('lib');
});
