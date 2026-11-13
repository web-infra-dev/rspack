import { value } from './value';

it('writes through the owner context after a custom runtime hot update', async () => {
  const runtimeRequire = __webpack_require__.getRuntimeRequire();
  const readBeforeUpdate = __webpack_require__.getRuntimePath;
  const writeBeforeUpdate = __webpack_require__.setRuntimePath;
  expect(runtimeRequire).not.toBe(__webpack_require__);
  expect(value).toBe(1);
  expect(__webpack_require__.customUpdate).toBe('generation-1');
  expect(__webpack_nonce__).toBe('hook-1');
  await NEXT_HMR();
  expect(value).toBe(2);
  expect(__webpack_require__.customUpdate).toBe('generation-2');
  expect(__webpack_nonce__).toBe('hook-2');
  expect(__webpack_require__.getRuntimeRequire()).toBe(runtimeRequire);
  writeBeforeUpdate('/from-old-runtime/');
  expect(__webpack_public_path__).toBe('/from-old-runtime/');
  expect(__webpack_require__.getRuntimePath()).toBe('/from-old-runtime/');
  __webpack_public_path__ = '/from-module/';
  expect(readBeforeUpdate()).toBe('/from-module/');
  expect(__webpack_require__.getRuntimePath()).toBe('/from-module/');
  __webpack_require__.setRuntimePath('/from-updated-runtime/');
  expect(readBeforeUpdate()).toBe('/from-updated-runtime/');
  expect(__webpack_public_path__).toBe('/from-updated-runtime/');
});

module.hot.accept('./value');
