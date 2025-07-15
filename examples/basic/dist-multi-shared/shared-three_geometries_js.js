"use strict";
(self["webpackChunkrspack_basic_example"] =
	self["webpackChunkrspack_basic_example"] || []).push([
	["shared-three_geometries_js"],
	{
		"./shared-three/geometries.js":
			/*!************************************!*\
  !*** ./shared-three/geometries.js ***!
  \************************************/
			function (
				__unused_webpack___webpack_module__,
				__webpack_exports__,
				__webpack_require__
			) {
				__webpack_require__.r(__webpack_exports__);
				__webpack_require__.d(__webpack_exports__, {
					GeometryFactory: (/* @common:endif */) =>
						/* @common:if [condition="treeShake.geometries.GeometryFactory"] */ GeometryFactory
				});
				class GeometryFactory {
					constructor(THREE) {
						this.THREE = THREE;
					}

					create(type) {
						switch (type) {
							case "box":
								return new this.THREE.BoxGeometry(1, 1, 1);
							case "sphere":
								return new this.THREE.SphereGeometry(0.5, 32, 16);
							case "cylinder":
								return new this.THREE.CylinderGeometry(0.5, 0.5, 1);
							case "torus":
								return new this.THREE.TorusGeometry(0.5, 0.2, 16, 32);
							case "knot":
								return new this.THREE.TorusKnotGeometry(0.5, 0.15, 100, 16);
							default:
								return new this.THREE.BoxGeometry(1, 1, 1);
						}
					}
				}
			}
	}
]);
