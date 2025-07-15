"use strict";
(self["webpackChunkrspack_basic_example"] =
	self["webpackChunkrspack_basic_example"] || []).push([
	["app3-game_src_index_js"],
	{
		"./app3-game/src/index.js":
			/*!********************************!*\
  !*** ./app3-game/src/index.js ***!
  \********************************/
			function (
				__unused_webpack___webpack_module__,
				__webpack_exports__,
				__webpack_require__
			) {
				var three__WEBPACK_IMPORTED_MODULE_0___namespace_cache;
				__webpack_require__.r(__webpack_exports__);
				__webpack_require__.d(__webpack_exports__, {
					GameEngine: () => GameEngine
				});
				/* ESM import */ var three__WEBPACK_IMPORTED_MODULE_0__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three */ "webpack/sharing/consume/default/three/three"
					);
				/* ESM import */ var three_examples_jsm_loaders_GLTFLoader_js__WEBPACK_IMPORTED_MODULE_1__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/loaders/GLTFLoader.js */ "webpack/sharing/consume/default/three/examples/jsm/loaders/GLTFLoader.js/three/examples/jsm/loaders/GLTFLoader.js"
					);
				/* ESM import */ var three_examples_jsm_loaders_DRACOLoader_js__WEBPACK_IMPORTED_MODULE_2__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/loaders/DRACOLoader.js */ "webpack/sharing/consume/default/three/examples/jsm/loaders/DRACOLoader.js/three/examples/jsm/loaders/DRACOLoader.js"
					);
				/* ESM import */ var three_src_animation_AnimationMixer_js__WEBPACK_IMPORTED_MODULE_3__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/src/animation/AnimationMixer.js */ "webpack/sharing/consume/default/three/src/animation/AnimationMixer.js/three/src/animation/AnimationMixer.js"
					);
				/* ESM import */ var _shared_three_scene_utils_js__WEBPACK_IMPORTED_MODULE_4__ =
					/* #__PURE__ */ __webpack_require__(
						/*! ../../shared-three/scene-utils.js */ "webpack/sharing/consume/default/scene-utils/./shared-three/scene-utils.js"
					);
				/* ESM import */ var _shared_three_physics_js__WEBPACK_IMPORTED_MODULE_5__ =
					/* #__PURE__ */ __webpack_require__(
						/*! ../../shared-three/physics.js */ "webpack/sharing/consume/default/physics/./shared-three/physics.js"
					);

				class GameEngine {
					constructor() {
						this.scene = new three__WEBPACK_IMPORTED_MODULE_0__.Scene();
						this.camera =
							new three__WEBPACK_IMPORTED_MODULE_0__.PerspectiveCamera(
								60,
								window.innerWidth / window.innerHeight,
								0.1,
								1000
							);
						this.renderer =
							new three__WEBPACK_IMPORTED_MODULE_0__.WebGLRenderer();
						this.clock = new three__WEBPACK_IMPORTED_MODULE_0__.Clock();
						this.mixers = [];

						this.utils = (0,
						_shared_three_scene_utils_js__WEBPACK_IMPORTED_MODULE_4__.createSceneUtils)(
							/*#__PURE__*/ three__WEBPACK_IMPORTED_MODULE_0___namespace_cache ||
								(three__WEBPACK_IMPORTED_MODULE_0___namespace_cache =
									__webpack_require__.t(three__WEBPACK_IMPORTED_MODULE_0__, 2))
						);
						this.physics =
							new _shared_three_physics_js__WEBPACK_IMPORTED_MODULE_5__.PhysicsWorld(
								/*#__PURE__*/ three__WEBPACK_IMPORTED_MODULE_0___namespace_cache ||
									(three__WEBPACK_IMPORTED_MODULE_0___namespace_cache =
										__webpack_require__.t(
											three__WEBPACK_IMPORTED_MODULE_0__,
											2
										))
							);

						// Setup loaders
						this.gltfLoader =
							new three_examples_jsm_loaders_GLTFLoader_js__WEBPACK_IMPORTED_MODULE_1__.GLTFLoader();
						this.dracoLoader =
							new three_examples_jsm_loaders_DRACOLoader_js__WEBPACK_IMPORTED_MODULE_2__.DRACOLoader();
						this.gltfLoader.setDRACOLoader(this.dracoLoader);
					}

					loadModel(url) {
						return this.gltfLoader.loadAsync(url).then(gltf => {
							this.scene.add(gltf.scene);
							if (gltf.animations.length > 0) {
								const mixer =
									new three_src_animation_AnimationMixer_js__WEBPACK_IMPORTED_MODULE_3__.AnimationMixer(
										gltf.scene
									);
								this.mixers.push(mixer);
							}
							return gltf;
						});
					}

					update() {
						const delta = this.clock.getDelta();
						this.mixers.forEach(mixer => mixer.update(delta));
						this.physics.update(delta);
					}
				}

				console.log("App3: Game engine loaded");
			}
	}
]);
