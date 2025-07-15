"use strict";
(self["webpackChunkrspack_basic_example"] =
	self["webpackChunkrspack_basic_example"] || []).push([
	["app2-editor_src_index_js"],
	{
		"./app2-editor/src/index.js":
			/*!**********************************!*\
  !*** ./app2-editor/src/index.js ***!
  \**********************************/
			function (
				__unused_webpack___webpack_module__,
				__webpack_exports__,
				__webpack_require__
			) {
				var three__WEBPACK_IMPORTED_MODULE_0___namespace_cache;
				__webpack_require__.r(__webpack_exports__);
				__webpack_require__.d(__webpack_exports__, {
					Editor: () => Editor
				});
				/* ESM import */ var three__WEBPACK_IMPORTED_MODULE_0__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three */ "webpack/sharing/consume/default/three/three"
					);
				/* ESM import */ var three_examples_jsm_controls_TransformControls_js__WEBPACK_IMPORTED_MODULE_1__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/controls/TransformControls.js */ "webpack/sharing/consume/default/three/examples/jsm/controls/TransformControls.js/three/examples/jsm/controls/TransformControls.js"
					);
				/* ESM import */ var three_src_helpers_BoxHelper_js__WEBPACK_IMPORTED_MODULE_2__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/src/helpers/BoxHelper.js */ "webpack/sharing/consume/default/three/src/helpers/BoxHelper.js/three/src/helpers/BoxHelper.js"
					);
				/* ESM import */ var three_src_helpers_GridHelper_js__WEBPACK_IMPORTED_MODULE_3__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/src/helpers/GridHelper.js */ "webpack/sharing/consume/default/three/src/helpers/GridHelper.js/three/src/helpers/GridHelper.js"
					);
				/* ESM import */ var _shared_three_scene_utils_js__WEBPACK_IMPORTED_MODULE_4__ =
					/* #__PURE__ */ __webpack_require__(
						/*! ../../shared-three/scene-utils.js */ "webpack/sharing/consume/default/scene-utils/./shared-three/scene-utils.js"
					);
				/* ESM import */ var _shared_three_geometries_js__WEBPACK_IMPORTED_MODULE_5__ =
					/* #__PURE__ */ __webpack_require__(
						/*! ../../shared-three/geometries.js */ "webpack/sharing/consume/default/geometries/./shared-three/geometries.js"
					);

				class Editor {
					constructor() {
						this.scene = new three__WEBPACK_IMPORTED_MODULE_0__.Scene();
						this.camera =
							new three__WEBPACK_IMPORTED_MODULE_0__.PerspectiveCamera(
								50,
								window.innerWidth / window.innerHeight,
								0.1,
								1000
							);
						this.renderer =
							new three__WEBPACK_IMPORTED_MODULE_0__.WebGLRenderer();
						this.utils = (0,
						_shared_three_scene_utils_js__WEBPACK_IMPORTED_MODULE_4__.createSceneUtils)(
							/*#__PURE__*/ three__WEBPACK_IMPORTED_MODULE_0___namespace_cache ||
								(three__WEBPACK_IMPORTED_MODULE_0___namespace_cache =
									__webpack_require__.t(three__WEBPACK_IMPORTED_MODULE_0__, 2))
						);
						this.geometryFactory =
							new _shared_three_geometries_js__WEBPACK_IMPORTED_MODULE_5__.GeometryFactory(
								/*#__PURE__*/ three__WEBPACK_IMPORTED_MODULE_0___namespace_cache ||
									(three__WEBPACK_IMPORTED_MODULE_0___namespace_cache =
										__webpack_require__.t(
											three__WEBPACK_IMPORTED_MODULE_0__,
											2
										))
							);

						// Add editor helpers
						const grid =
							new three_src_helpers_GridHelper_js__WEBPACK_IMPORTED_MODULE_3__.GridHelper(
								20,
								20
							);
						this.scene.add(grid);

						this.transformControls =
							new three_examples_jsm_controls_TransformControls_js__WEBPACK_IMPORTED_MODULE_1__.TransformControls(
								this.camera,
								this.renderer.domElement
							);
						this.scene.add(this.transformControls);
					}

					addPrimitive(type) {
						const geometry = this.geometryFactory.create(type);
						const material =
							new three__WEBPACK_IMPORTED_MODULE_0__.MeshStandardMaterial();
						const mesh = new three__WEBPACK_IMPORTED_MODULE_0__.Mesh(
							geometry,
							material
						);
						this.scene.add(mesh);
						return mesh;
					}
				}

				console.log("App2: Editor loaded");
			}
	}
]);
