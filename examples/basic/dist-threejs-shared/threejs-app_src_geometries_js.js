"use strict";
(self["webpackChunkrspack_basic_example"] =
	self["webpackChunkrspack_basic_example"] || []).push([
	["threejs-app_src_geometries_js"],
	{
		"./threejs-app/src/geometries.js":
			/*!***************************************!*\
  !*** ./threejs-app/src/geometries.js ***!
  \***************************************/
			function (
				__unused_webpack___webpack_module__,
				__webpack_exports__,
				__webpack_require__
			) {
				__webpack_require__.r(__webpack_exports__);
				__webpack_require__.d(__webpack_exports__, {
					GeometryProcessor: () => GeometryProcessor,
					createGeometries: () => createGeometries
				});
				/* ESM import */ var three__WEBPACK_IMPORTED_MODULE_0__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three */ "webpack/sharing/consume/default/three/three"
					);

				function createGeometries() {
					return {
						box: new three__WEBPACK_IMPORTED_MODULE_0__.BoxGeometry(1, 1, 1),
						sphere: new three__WEBPACK_IMPORTED_MODULE_0__.SphereGeometry(
							1,
							32,
							32
						),
						cylinder: new three__WEBPACK_IMPORTED_MODULE_0__.CylinderGeometry(
							1,
							1,
							2,
							32
						),
						torus: new three__WEBPACK_IMPORTED_MODULE_0__.TorusGeometry(
							1,
							0.4,
							16,
							100
						),
						torusKnot: new three__WEBPACK_IMPORTED_MODULE_0__.TorusKnotGeometry(
							1,
							0.3,
							100,
							16
						),
						plane: new three__WEBPACK_IMPORTED_MODULE_0__.PlaneGeometry(5, 5),
						cone: new three__WEBPACK_IMPORTED_MODULE_0__.ConeGeometry(1, 2, 32),
						dodecahedron:
							new three__WEBPACK_IMPORTED_MODULE_0__.DodecahedronGeometry(1),
						icosahedron:
							new three__WEBPACK_IMPORTED_MODULE_0__.IcosahedronGeometry(1),
						octahedron:
							new three__WEBPACK_IMPORTED_MODULE_0__.OctahedronGeometry(1),
						tetrahedron:
							new three__WEBPACK_IMPORTED_MODULE_0__.TetrahedronGeometry(1)
					};
				}

				class GeometryProcessor {
					static merge(geometries) {
						const merged =
							new three__WEBPACK_IMPORTED_MODULE_0__.BufferGeometry();
						// Simplified merge logic
						return merged;
					}

					static optimize(geometry) {
						geometry.computeVertexNormals();
						geometry.computeBoundingBox();
						geometry.computeBoundingSphere();
						return geometry;
					}
				}
			}
	}
]);
