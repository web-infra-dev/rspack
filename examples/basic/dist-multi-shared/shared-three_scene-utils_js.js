"use strict";
(self["webpackChunkrspack_basic_example"] =
	self["webpackChunkrspack_basic_example"] || []).push([
	["shared-three_scene-utils_js"],
	{
		"./shared-three/scene-utils.js":
			/*!*************************************!*\
  !*** ./shared-three/scene-utils.js ***!
  \*************************************/
			function (
				__unused_webpack___webpack_module__,
				__webpack_exports__,
				__webpack_require__
			) {
				__webpack_require__.r(__webpack_exports__);
				__webpack_require__.d(__webpack_exports__, {
					createSceneUtils: (/* @common:endif */) =>
						/* @common:if [condition="treeShake.scene-utils.createSceneUtils"] */ createSceneUtils
				});
				function createSceneUtils(THREE) {
					return {
						randomGeometry() {
							const geometries = [
								new THREE.BoxGeometry(1, 1, 1),
								new THREE.SphereGeometry(0.5, 32, 16),
								new THREE.CylinderGeometry(0.5, 0.5, 1, 32),
								new THREE.TorusGeometry(0.5, 0.2, 16, 32),
								new THREE.ConeGeometry(0.5, 1, 32)
							];
							return geometries[Math.floor(Math.random() * geometries.length)];
						},

						createLighting() {
							const lights = [];
							lights.push(new THREE.AmbientLight(0x404040));
							lights.push(new THREE.DirectionalLight(0xffffff, 1));
							lights.push(new THREE.PointLight(0xff0000, 1, 100));
							return lights;
						}
					};
				}
			}
	}
]);
