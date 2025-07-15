"use strict";
(self["webpackChunkrspack_basic_example"] =
	self["webpackChunkrspack_basic_example"] || []).push([
	["shared-three_physics_js"],
	{
		"./shared-three/physics.js":
			/*!*********************************!*\
  !*** ./shared-three/physics.js ***!
  \*********************************/
			function (
				__unused_webpack___webpack_module__,
				__webpack_exports__,
				__webpack_require__
			) {
				__webpack_require__.r(__webpack_exports__);
				__webpack_require__.d(__webpack_exports__, {
					PhysicsWorld: (/* @common:endif */) =>
						/* @common:if [condition="treeShake.physics.PhysicsWorld"] */ PhysicsWorld
				});
				class PhysicsWorld {
					constructor(THREE) {
						this.THREE = THREE;
						this.bodies = [];
						this.gravity = new THREE.Vector3(0, -9.8, 0);
					}

					addBody(mesh, mass = 1) {
						this.bodies.push({
							mesh,
							mass,
							velocity: new this.THREE.Vector3()
						});
					}

					update(delta) {
						this.bodies.forEach(body => {
							body.velocity.add(this.gravity.clone().multiplyScalar(delta));
							body.mesh.position.add(
								body.velocity.clone().multiplyScalar(delta)
							);
						});
					}
				}
			}
	}
]);
