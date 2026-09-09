export let [rspackExports, __webpack_exports__] = [40, 42];
rspackExports += 1;
__webpack_exports__ += 1;
[rspackExports, __webpack_exports__] = [rspackExports + 2, __webpack_exports__ + 2];

export function update() {
  ({ rspackExports, __webpack_exports__ } = {
    rspackExports: rspackExports + 1,
    __webpack_exports__: __webpack_exports__ + 1,
  });
}
