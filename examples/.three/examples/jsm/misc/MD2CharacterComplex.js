/**
 * @author alteredq / http://alteredqualia.com/
 */

import {
	Box3,
	Math as _Math,
	MeshLambertMaterial,
	Object3D,
	TextureLoader,
	UVMapping
} from "../../../build/three.module.js";
import { MD2Loader } from "../loaders/MD2Loader.js";
import { MorphBlendMesh } from "../misc/MorphBlendMesh.js";

var MD2CharacterComplex = function () {

	var scope = this;

	this.scale = 1;

	// animation parameters

	this.animationFPS = 6;
	this.transitionFrames = 15;

	// movement model parameters

	this.maxSpeed = 275;
	this.maxReverseSpeed = - 275;

	this.frontAcceleration = 600;
	this.backAcceleration = 600;

	this.frontDecceleration = 600;

	this.angularSpeed = 2.5;

	// rig

	this.root = new Object3D();

	this.meshBody = null;
	this.meshWeapon = null;

	this.controls = null;

	// skins

	this.skinsBody = [];
	this.skinsWeapon = [];

	this.weapons = [];

	this.currentSkin = undefined;

	//

	this.onLoadComplete = function () {};

	// internals

	this.meshes = [];
	this.animations = {};

	this.loadCounter = 0;

	// internal movement control variables

	this.speed = 0;
	this.bodyOrientation = 0;

	this.walkSpeed = this.maxSpeed;
	this.crouchSpeed = this.maxSpeed * 0.5;

	// internal animation parameters

	this.activeAnimation = null;
	this.oldAnimation = null;

	// API

	this.enableShadows = function ( enable ) {

		for ( var i = 0; i < this.meshes.length; i ++ ) {

			this.meshes[ i ].castShadow = enable;
			this.meshes[ i ].receiveShadow = enable;

		}

	};

	this.setVisible = function ( enable ) {

		for ( var i = 0; i < this.meshes.length; i ++ ) {

			this.meshes[ i ].visible = enable;
			this.meshes[ i ].visible = enable;

		}

	};


	this.shareParts = function ( original ) {

		this.animations = original.animations;
		this.walkSpeed = original.walkSpeed;
		this.crouchSpeed = original.crouchSpeed;

		this.skinsBody = original.skinsBody;
		this.skinsWeapon = original.skinsWeapon;

		// BODY

		var mesh = createPart( original.meshBody.geometry, this.skinsBody[ 0 ] );
		mesh.scale.set( this.scale, this.scale, this.scale );

		this.root.position.y = original.root.position.y;
		this.root.add( mesh );

		this.meshBody = mesh;

		this.meshes.push( mesh );

		// WEAPONS

		for ( var i = 0; i < original.weapons.length; i ++ ) {

			var meshWeapon = createPart( original.weapons[ i ].geometry, this.skinsWeapon[ i ] );
			meshWeapon.scale.set( this.scale, this.scale, this.scale );
			meshWeapon.visible = false;

			meshWeapon.name = original.weapons[ i ].name;

			this.root.add( meshWeapon );

			this.weapons[ i ] = meshWeapon;
			this.meshWeapon = meshWeapon;

			this.meshes.push( meshWeapon );

		}

	};

	this.loadParts = function ( config ) {

		this.animations = config.animations;
		this.walkSpeed = config.walkSpeed;
		this.crouchSpeed = config.crouchSpeed;

		this.loadCounter = config.weapons.length * 2 + config.skins.length + 1;

		var weaponsTextures = [];
		for ( var i = 0; i < config.weapons.length; i ++ ) weaponsTextures[ i ] = config.weapons[ i ][ 1 ];

		// SKINS

		this.skinsBody = loadTextures( config.baseUrl + "skins/", config.skins );
		this.skinsWeapon = loadTextures( config.baseUrl + "skins/", weaponsTextures );

		// BODY

		var loader = new MD2Loader();

		loader.load( config.baseUrl + config.body, function ( geo ) {

			var boundingBox = new Box3();
			boundingBox.setFromBufferAttribute( geo.attributes.position );

			scope.root.position.y = - scope.scale * boundingBox.min.y;

			var mesh = createPart( geo, scope.skinsBody[ 0 ] );
			mesh.scale.set( scope.scale, scope.scale, scope.scale );

			scope.root.add( mesh );

			scope.meshBody = mesh;
			scope.meshes.push( mesh );

			checkLoadingComplete();

		} );

		// WEAPONS

		var generateCallback = function ( index, name ) {

			return function ( geo ) {

				var mesh = createPart( geo, scope.skinsWeapon[ index ] );
				mesh.scale.set( scope.scale, scope.scale, scope.scale );
				mesh.visible = false;

				mesh.name = name;

				scope.root.add( mesh );

				scope.weapons[ index ] = mesh;
				scope.meshWeapon = mesh;
				scope.meshes.push( mesh );

				checkLoadingComplete();

			};

		};

		for ( var i = 0; i < config.weapons.length; i ++ ) {

			loader.load( config.baseUrl + config.weapons[ i ][ 0 ], generateCallback( i, config.weapons[ i ][ 0 ] ) );

		}

	};

	this.setPlaybackRate = function ( rate ) {

		if ( this.meshBody ) this.meshBody.duration = this.meshBody.baseDuration / rate;
		if ( this.meshWeapon ) this.meshWeapon.duration = this.meshWeapon.baseDuration / rate;

	};

	this.setWireframe = function ( wireframeEnabled ) {

		if ( wireframeEnabled ) {

			if ( this.meshBody ) this.meshBody.material = this.meshBody.materialWireframe;
			if ( this.meshWeapon ) this.meshWeapon.material = this.meshWeapon.materialWireframe;

		} else {

			if ( this.meshBody ) this.meshBody.material = this.meshBody.materialTexture;
			if ( this.meshWeapon ) this.meshWeapon.material = this.meshWeapon.materialTexture;

		}

	};

	this.setSkin = function ( index ) {

		if ( this.meshBody && this.meshBody.material.wireframe === false ) {

			this.meshBody.material.map = this.skinsBody[ index ];
			this.currentSkin = index;

		}

	};

	this.setWeapon = function ( index ) {

		for ( var i = 0; i < this.weapons.length; i ++ ) this.weapons[ i ].visible = false;

		var activeWeapon = this.weapons[ index ];

		if ( activeWeapon ) {

			activeWeapon.visible = true;
			this.meshWeapon = activeWeapon;

			if ( this.activeAnimation ) {

				activeWeapon.playAnimation( this.activeAnimation );
				this.meshWeapon.setAnimationTime( this.activeAnimation, this.meshBody.getAnimationTime( this.activeAnimation ) );

			}

		}

	};

	this.setAnimation = function ( animationName ) {

		if ( animationName === this.activeAnimation || ! animationName ) return;

		if ( this.meshBody ) {

			this.meshBody.setAnimationWeight( animationName, 0 );
			this.meshBody.playAnimation( animationName );

			this.oldAnimation = this.activeAnimation;
			this.activeAnimation = animationName;

			this.blendCounter = this.transitionFrames;

		}

		if ( this.meshWeapon ) {

			this.meshWeapon.setAnimationWeight( animationName, 0 );
			this.meshWeapon.playAnimation( animationName );

		}


	};

	this.update = function ( delta ) {

		if ( this.controls ) this.updateMovementModel( delta );

		if ( this.animations ) {

			this.updateBehaviors();
			this.updateAnimations( delta );

		}

	};

	this.updateAnimations = function ( delta ) {

		var mix = 1;

		if ( this.blendCounter > 0 ) {

			mix = ( this.transitionFrames - this.blendCounter ) / this.transitionFrames;
			this.blendCounter -= 1;

		}

		if ( this.meshBody ) {

			this.meshBody.update( delta );

			this.meshBody.setAnimationWeight( this.activeAnimation, mix );
			this.meshBody.setAnimationWeight( this.oldAnimation, 1 - mix );

		}

		if ( this.meshWeapon ) {

			this.meshWeapon.update( delta );

			this.meshWeapon.setAnimationWeight( this.activeAnimation, mix );
			this.meshWeapon.setAnimationWeight( this.oldAnimation, 1 - mix );

		}

	};

	this.updateBehaviors = function () {

		var controls = this.controls;
		var animations = this.animations;

		var moveAnimation, idleAnimation;

		// crouch vs stand

		if ( controls.crouch ) {

			moveAnimation = animations[ "crouchMove" ];
			idleAnimation = animations[ "crouchIdle" ];

		} else {

			moveAnimation = animations[ "move" ];
			idleAnimation = animations[ "idle" ];

		}

		// actions

		if ( controls.jump ) {

			moveAnimation = animations[ "jump" ];
			idleAnimation = animations[ "jump" ];

		}

		if ( controls.attack ) {

			if ( controls.crouch ) {

				moveAnimation = animations[ "crouchAttack" ];
				idleAnimation = animations[ "crouchAttack" ];

			} else {

				moveAnimation = animations[ "attack" ];
				idleAnimation = animations[ "attack" ];

			}

		}

		// set animations

		if ( controls.moveForward || controls.moveBackward || controls.moveLeft || controls.moveRight ) {

			if ( this.activeAnimation !== moveAnimation ) {

				this.setAnimation( moveAnimation );

			}

		}


		if ( Math.abs( this.speed ) < 0.2 * this.maxSpeed && ! ( controls.moveLeft || controls.moveRight || controls.moveForward || controls.moveBackward ) ) {

			if ( this.activeAnimation !== idleAnimation ) {

				this.setAnimation( idleAnimation );

			}

		}

		// set animation direction

		if ( controls.moveForward ) {

			if ( this.meshBody ) {

				this.meshBody.setAnimationDirectionForward( this.activeAnimation );
				this.meshBody.setAnimationDirectionForward( this.oldAnimation );

			}

			if ( this.meshWeapon ) {

				this.meshWeapon.setAnimationDirectionForward( this.activeAnimation );
				this.meshWeapon.setAnimationDirectionForward( this.oldAnimation );

			}

		}

		if ( controls.moveBackward ) {

			if ( this.meshBody ) {

				this.meshBody.setAnimationDirectionBackward( this.activeAnimation );
				this.meshBody.setAnimationDirectionBackward( this.oldAnimation );

			}

			if ( this.meshWeapon ) {

				this.meshWeapon.setAnimationDirectionBackward( this.activeAnimation );
				this.meshWeapon.setAnimationDirectionBackward( this.oldAnimation );

			}

		}

	};

	this.updateMovementModel = function ( delta ) {

		var controls = this.controls;

		// speed based on controls

		if ( controls.crouch ) 	this.maxSpeed = this.crouchSpeed;
		else this.maxSpeed = this.walkSpeed;

		this.maxReverseSpeed = - this.maxSpeed;

		if ( controls.moveForward ) this.speed = _Math.clamp( this.speed + delta * this.frontAcceleration, this.maxReverseSpeed, this.maxSpeed );
		if ( controls.moveBackward ) this.speed = _Math.clamp( this.speed - delta * this.backAcceleration, this.maxReverseSpeed, this.maxSpeed );

		// orientation based on controls
		// (don't just stand while turning)

		var dir = 1;

		if ( controls.moveLeft ) {

			this.bodyOrientation += delta * this.angularSpeed;
			this.speed = _Math.clamp( this.speed + dir * delta * this.frontAcceleration, this.maxReverseSpeed, this.maxSpeed );

		}

		if ( controls.moveRight ) {

			this.bodyOrientation -= delta * this.angularSpeed;
			this.speed = _Math.clamp( this.speed + dir * delta * this.frontAcceleration, this.maxReverseSpeed, this.maxSpeed );

		}

		// speed decay

		if ( ! ( controls.moveForward || controls.moveBackward ) ) {

			if ( this.speed > 0 ) {

				var k = exponentialEaseOut( this.speed / this.maxSpeed );
				this.speed = _Math.clamp( this.speed - k * delta * this.frontDecceleration, 0, this.maxSpeed );

			} else {

				var k = exponentialEaseOut( this.speed / this.maxReverseSpeed );
				this.speed = _Math.clamp( this.speed + k * delta * this.backAcceleration, this.maxReverseSpeed, 0 );

			}

		}

		// displacement

		var forwardDelta = this.speed * delta;

		this.root.position.x += Math.sin( this.bodyOrientation ) * forwardDelta;
		this.root.position.z += Math.cos( this.bodyOrientation ) * forwardDelta;

		// steering

		this.root.rotation.y = this.bodyOrientation;

	};

	// internal helpers

	function loadTextures( baseUrl, textureUrls ) {

		var textureLoader = new TextureLoader();
		var textures = [];

		for ( var i = 0; i < textureUrls.length; i ++ ) {

			textures[ i ] = textureLoader.load( baseUrl + textureUrls[ i ], checkLoadingComplete );
			textures[ i ].mapping = UVMapping;
			textures[ i ].name = textureUrls[ i ];

		}

		return textures;

	}

	function createPart( geometry, skinMap ) {

		var materialWireframe = new MeshLambertMaterial( { color: 0xffaa00, wireframe: true, morphTargets: true, morphNormals: true } );
		var materialTexture = new MeshLambertMaterial( { color: 0xffffff, wireframe: false, map: skinMap, morphTargets: true, morphNormals: true } );

		//

		var mesh = new MorphBlendMesh( geometry, materialTexture );
		mesh.rotation.y = - Math.PI / 2;

		//

		mesh.materialTexture = materialTexture;
		mesh.materialWireframe = materialWireframe;

		//

		mesh.autoCreateAnimations( scope.animationFPS );

		return mesh;

	}

	function checkLoadingComplete() {

		scope.loadCounter -= 1;
		if ( scope.loadCounter === 0 ) 	scope.onLoadComplete();

	}

	function exponentialEaseOut( k ) {

		return k === 1 ? 1 : - Math.pow( 2, - 10 * k ) + 1;

	}

};

export { MD2CharacterComplex };
