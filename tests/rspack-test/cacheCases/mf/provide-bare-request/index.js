import value from 'shared-package';
import phase from './phase';

it('registers bare-request providers after restoring the persistent cache', async () => {
  expect(value).toBe('exact provider');
  expect(phase).toBe(COMPILER_INDEX);
  await __webpack_init_sharing__('default');
  const scope = __webpack_share_scopes__.default;
  expect(Object.keys(scope).sort()).toEqual(['exact', 'included', 'prefix/feature']);
  const exact = await scope.exact['1.0.0'].get();
  const prefix = await scope['prefix/feature']['1.0.0'].get();
  expect(exact()).toBe('exact provider');
  expect(prefix()).toBe('prefix provider');
  if (COMPILER_INDEX === 0) await NEXT_START();
});
