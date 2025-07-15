"use strict";
(self["webpackChunkrspack_basic_example"] =
	self["webpackChunkrspack_basic_example"] || []).push([
	["threejs-app_src_main_js"],
	{
		"./threejs-app/src/main.js":
			/*!*********************************!*\
  !*** ./threejs-app/src/main.js ***!
  \*********************************/
			function (
				__unused_webpack___webpack_module__,
				__webpack_exports__,
				__webpack_require__
			) {
				var three__WEBPACK_IMPORTED_MODULE_0___namespace_cache;
				__webpack_require__.r(__webpack_exports__);
				__webpack_require__.d(__webpack_exports__, {
					DRACOLoader: () =>
						/* reexport safe */ three_examples_jsm_loaders_DRACOLoader_js__WEBPACK_IMPORTED_MODULE_3__.DRACOLoader,
					GLTFLoader: () =>
						/* reexport safe */ three_examples_jsm_loaders_GLTFLoader_js__WEBPACK_IMPORTED_MODULE_2__.GLTFLoader,
					RGBELoader: () =>
						/* reexport safe */ three_examples_jsm_loaders_RGBELoader_js__WEBPACK_IMPORTED_MODULE_4__.RGBELoader,
					THREE: () =>
						/* reexport fake namespace object from non-ESM */ three__WEBPACK_IMPORTED_MODULE_0___namespace_cache ||
						(three__WEBPACK_IMPORTED_MODULE_0___namespace_cache =
							__webpack_require__.t(three__WEBPACK_IMPORTED_MODULE_0__, 2)),
					createScene: () => createScene
				});
				/* ESM import */ var three__WEBPACK_IMPORTED_MODULE_0__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three */ "webpack/sharing/consume/default/three/three"
					);
				/* ESM import */ var three_examples_jsm_controls_OrbitControls_js__WEBPACK_IMPORTED_MODULE_1__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/controls/OrbitControls.js */ "webpack/sharing/consume/default/three-orbit-controls/three/examples/jsm/controls/OrbitControls.js"
					);
				/* ESM import */ var three_examples_jsm_loaders_GLTFLoader_js__WEBPACK_IMPORTED_MODULE_2__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/loaders/GLTFLoader.js */ "webpack/sharing/consume/default/three-gltf-loader/three/examples/jsm/loaders/GLTFLoader.js"
					);
				/* ESM import */ var three_examples_jsm_loaders_DRACOLoader_js__WEBPACK_IMPORTED_MODULE_3__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/loaders/DRACOLoader.js */ "webpack/sharing/consume/default/three-draco-loader/three/examples/jsm/loaders/DRACOLoader.js"
					);
				/* ESM import */ var three_examples_jsm_loaders_RGBELoader_js__WEBPACK_IMPORTED_MODULE_4__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/loaders/RGBELoader.js */ "webpack/sharing/consume/default/three-rgbe-loader/three/examples/jsm/loaders/RGBELoader.js"
					);
				/* ESM import */ var three_examples_jsm_postprocessing_EffectComposer_js__WEBPACK_IMPORTED_MODULE_5__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/postprocessing/EffectComposer.js */ "webpack/sharing/consume/default/three-effect-composer/three/examples/jsm/postprocessing/EffectComposer.js"
					);
				/* ESM import */ var three_examples_jsm_postprocessing_RenderPass_js__WEBPACK_IMPORTED_MODULE_6__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/postprocessing/RenderPass.js */ "webpack/sharing/consume/default/three-render-pass/three/examples/jsm/postprocessing/RenderPass.js"
					);
				/* ESM import */ var three_examples_jsm_postprocessing_UnrealBloomPass_js__WEBPACK_IMPORTED_MODULE_7__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/postprocessing/UnrealBloomPass.js */ "webpack/sharing/consume/default/three-bloom-pass/three/examples/jsm/postprocessing/UnrealBloomPass.js"
					);

				console.log(
					"Three.js version:",
					three__WEBPACK_IMPORTED_MODULE_0__.REVISION
				);

				// Create a scene
				function createScene() {
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

					renderer.setSize(window.innerWidth, window.innerHeight);

					// Add lights
					const ambientLight =
						new three__WEBPACK_IMPORTED_MODULE_0__.AmbientLight(0x404040);
					scene.add(ambientLight);

					const directionalLight =
						new three__WEBPACK_IMPORTED_MODULE_0__.DirectionalLight(
							0xffffff,
							1
						);
					directionalLight.position.set(5, 5, 5);
					scene.add(directionalLight);

					// Add geometry
					const geometry = new three__WEBPACK_IMPORTED_MODULE_0__.BoxGeometry(
						1,
						1,
						1
					);
					const material =
						new three__WEBPACK_IMPORTED_MODULE_0__.MeshPhongMaterial({
							color: 0x00ff00
						});
					const cube = new three__WEBPACK_IMPORTED_MODULE_0__.Mesh(
						geometry,
						material
					);
					scene.add(cube);

					camera.position.z = 5;

					// Add controls
					const controls =
						new three_examples_jsm_controls_OrbitControls_js__WEBPACK_IMPORTED_MODULE_1__.OrbitControls(
							camera,
							renderer.domElement
						);

					// Add post-processing
					const composer =
						new three_examples_jsm_postprocessing_EffectComposer_js__WEBPACK_IMPORTED_MODULE_5__.EffectComposer(
							renderer
						);
					const renderPass =
						new three_examples_jsm_postprocessing_RenderPass_js__WEBPACK_IMPORTED_MODULE_6__.RenderPass(
							scene,
							camera
						);
					composer.addPass(renderPass);

					const bloomPass =
						new three_examples_jsm_postprocessing_UnrealBloomPass_js__WEBPACK_IMPORTED_MODULE_7__.UnrealBloomPass(
							new three__WEBPACK_IMPORTED_MODULE_0__.Vector2(
								window.innerWidth,
								window.innerHeight
							),
							1.5,
							0.4,
							0.85
						);
					composer.addPass(bloomPass);

					return { scene, camera, renderer, cube, controls, composer };
				}

				// Export loaders for use in other modules
			}
	}
]);
