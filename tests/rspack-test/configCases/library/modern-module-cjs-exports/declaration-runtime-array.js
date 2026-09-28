export let [exports, __webpack_exports__] = [40, 42];
exports += 1;
__webpack_exports__ += 1;
[exports, __webpack_exports__] = [exports + 2, __webpack_exports__ + 2];

export function update() {
  ({ exports, __webpack_exports__ } = {
    exports: exports + 1,
    __webpack_exports__: __webpack_exports__ + 1,
  });
}
