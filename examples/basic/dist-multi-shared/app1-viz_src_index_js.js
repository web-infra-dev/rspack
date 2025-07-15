"use strict";
(self["webpackChunkrspack_basic_example"] =
	self["webpackChunkrspack_basic_example"] || []).push([
	["app1-viz_src_index_js"],
	{
		"./app1-viz/src/index.js":
			/*!*******************************!*\
  !*** ./app1-viz/src/index.js ***!
  \*******************************/
			function (
				__unused_webpack___webpack_module__,
				__webpack_exports__,
				__webpack_require__
			) {
				var three__WEBPACK_IMPORTED_MODULE_0___namespace_cache;
				__webpack_require__.r(__webpack_exports__);
				__webpack_require__.d(__webpack_exports__, {
					createVisualization: () => createVisualization
				});
				/* ESM import */ var three__WEBPACK_IMPORTED_MODULE_0__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three */ "webpack/sharing/consume/default/three/three"
					);
				/* ESM import */ var three_examples_jsm_controls_OrbitControls_js__WEBPACK_IMPORTED_MODULE_1__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/controls/OrbitControls.js */ "webpack/sharing/consume/default/three/examples/jsm/controls/OrbitControls.js/three/examples/jsm/controls/OrbitControls.js"
					);
				/* ESM import */ var three_examples_jsm_postprocessing_EffectComposer_js__WEBPACK_IMPORTED_MODULE_2__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/postprocessing/EffectComposer.js */ "webpack/sharing/consume/default/three/examples/jsm/postprocessing/EffectComposer.js/three/examples/jsm/postprocessing/EffectComposer.js"
					);
				/* ESM import */ var three_examples_jsm_postprocessing_RenderPass_js__WEBPACK_IMPORTED_MODULE_3__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/postprocessing/RenderPass.js */ "webpack/sharing/consume/default/three/examples/jsm/postprocessing/RenderPass.js/three/examples/jsm/postprocessing/RenderPass.js"
					);
				/* ESM import */ var _shared_three_scene_utils_js__WEBPACK_IMPORTED_MODULE_4__ =
					/* #__PURE__ */ __webpack_require__(
						/*! ../../shared-three/scene-utils.js */ "webpack/sharing/consume/default/scene-utils/./shared-three/scene-utils.js"
					);
				/* ESM import */ var _shared_three_materials_js__WEBPACK_IMPORTED_MODULE_5__ =
					/* #__PURE__ */ __webpack_require__(
						/*! ../../shared-three/materials.js */ "webpack/sharing/consume/default/materials/./shared-three/materials.js"
					);

				function createVisualization() {
					const scene = new three__WEBPACK_IMPORTED_MODULE_0__.Scene();
					const camera =
						new three__WEBPACK_IMPORTED_MODULE_0__.PerspectiveCamera(
							75,
							window.innerWidth / window.innerHeight,
							0.1,
							1000
						);
					const renderer =
						new three__WEBPACK_IMPORTED_MODULE_0__.WebGLRenderer();

					const utils = (0,
					_shared_three_scene_utils_js__WEBPACK_IMPORTED_MODULE_4__.createSceneUtils)(
						/*#__PURE__*/ three__WEBPACK_IMPORTED_MODULE_0___namespace_cache ||
							(three__WEBPACK_IMPORTED_MODULE_0___namespace_cache =
								__webpack_require__.t(three__WEBPACK_IMPORTED_MODULE_0__, 2))
					);
					const materials =
						new _shared_three_materials_js__WEBPACK_IMPORTED_MODULE_5__.MaterialLibrary(
							/*#__PURE__*/ three__WEBPACK_IMPORTED_MODULE_0___namespace_cache ||
								(three__WEBPACK_IMPORTED_MODULE_0___namespace_cache =
									__webpack_require__.t(three__WEBPACK_IMPORTED_MODULE_0__, 2))
						);

					// Create complex scene
					for (let i = 0; i < 20; i++) {
						const geometry = utils.randomGeometry();
						const material = materials.getRandom();
						const mesh = new three__WEBPACK_IMPORTED_MODULE_0__.Mesh(
							geometry,
							material
						);
						mesh.position.random().multiplyScalar(10);
						scene.add(mesh);
					}

					return { scene, camera, renderer };
				}

				console.log("App1: Visualization loaded");
			}
	}
]);
