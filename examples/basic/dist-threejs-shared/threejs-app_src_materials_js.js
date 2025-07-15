"use strict";
(self["webpackChunkrspack_basic_example"] =
	self["webpackChunkrspack_basic_example"] || []).push([
	["threejs-app_src_materials_js"],
	{
		"./threejs-app/src/materials.js":
			/*!**************************************!*\
  !*** ./threejs-app/src/materials.js ***!
  \**************************************/
			function (
				__unused_webpack___webpack_module__,
				__webpack_exports__,
				__webpack_require__
			) {
				var three__WEBPACK_IMPORTED_MODULE_0___namespace_cache;
				__webpack_require__.r(__webpack_exports__);
				__webpack_require__.d(__webpack_exports__, {
					MaterialFactory: () => MaterialFactory,
					createAdvancedMaterials: () => createAdvancedMaterials
				});
				/* ESM import */ var three__WEBPACK_IMPORTED_MODULE_0__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three */ "webpack/sharing/consume/default/three/three"
					);

				function createAdvancedMaterials() {
					return {
						standard:
							new three__WEBPACK_IMPORTED_MODULE_0__.MeshStandardMaterial({
								color: 0x2194ce,
								roughness: 0.5,
								metalness: 0.5
							}),
						physical:
							new three__WEBPACK_IMPORTED_MODULE_0__.MeshPhysicalMaterial({
								color: 0xff0000,
								roughness: 0.2,
								metalness: 0.8,
								clearcoat: 1.0,
								clearcoatRoughness: 0.1
							}),
						toon: new three__WEBPACK_IMPORTED_MODULE_0__.MeshToonMaterial({
							color: 0x00ff00
						}),
						lambert: new three__WEBPACK_IMPORTED_MODULE_0__.MeshLambertMaterial(
							{
								color: 0x0000ff
							}
						)
					};
				}

				class MaterialFactory {
					constructor() {
						this.cache = new Map();
					}

					getMaterial(type, options) {
						const key = `${type}_${JSON.stringify(options)}`;
						if (!this.cache.has(key)) {
							this.cache.set(
								key,
								new /*#__PURE__*/ (three__WEBPACK_IMPORTED_MODULE_0___namespace_cache ||
									(three__WEBPACK_IMPORTED_MODULE_0___namespace_cache =
										__webpack_require__.t(
											three__WEBPACK_IMPORTED_MODULE_0__,
											2
										)))[type](options)
							);
						}
						return this.cache.get(key);
					}
				}
			}
	}
]);
