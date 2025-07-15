"use strict";
(self["webpackChunkrspack_basic_example"] =
	self["webpackChunkrspack_basic_example"] || []).push([
	["shared-three_materials_js"],
	{
		"./shared-three/materials.js":
			/*!***********************************!*\
  !*** ./shared-three/materials.js ***!
  \***********************************/
			function (
				__unused_webpack___webpack_module__,
				__webpack_exports__,
				__webpack_require__
			) {
				__webpack_require__.r(__webpack_exports__);
				__webpack_require__.d(__webpack_exports__, {
					MaterialLibrary: (/* @common:endif */) =>
						/* @common:if [condition="treeShake.materials.MaterialLibrary"] */ MaterialLibrary
				});
				class MaterialLibrary {
					constructor(THREE) {
						this.THREE = THREE;
						this.materials = this.createMaterials();
					}

					createMaterials() {
						return [
							new this.THREE.MeshBasicMaterial({ color: 0xff0000 }),
							new this.THREE.MeshStandardMaterial({
								color: 0x00ff00,
								roughness: 0.5
							}),
							new this.THREE.MeshPhongMaterial({ color: 0x0000ff }),
							new this.THREE.MeshPhysicalMaterial({
								color: 0xffffff,
								metalness: 1,
								roughness: 0
							}),
							new this.THREE.MeshToonMaterial({ color: 0xff00ff })
						];
					}

					getRandom() {
						return this.materials[
							Math.floor(Math.random() * this.materials.length)
						];
					}
				}
			}
	}
]);
