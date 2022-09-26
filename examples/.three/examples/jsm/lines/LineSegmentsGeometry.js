/**
 * @author WestLangley / http://github.com/WestLangley
 *
 */

import {
	Box3,
	Float32BufferAttribute,
	InstancedBufferGeometry,
	InstancedInterleavedBuffer,
	InterleavedBufferAttribute,
	Sphere,
	Vector3,
	WireframeGeometry
} from "../../../build/three.module.js";

var LineSegmentsGeometry = function () {

	InstancedBufferGeometry.call( this );

	this.type = 'LineSegmentsGeometry';

	var positions = [ - 1, 2, 0, 1, 2, 0, - 1, 1, 0, 1, 1, 0, - 1, 0, 0, 1, 0, 0, - 1, - 1, 0, 1, - 1, 0 ];
	var uvs = [ - 1, 2, 1, 2, - 1, 1, 1, 1, - 1, - 1, 1, - 1, - 1, - 2, 1, - 2 ];
	var index = [ 0, 2, 1, 2, 3, 1, 2, 4, 3, 4, 5, 3, 4, 6, 5, 6, 7, 5 ];

	this.setIndex( index );
	this.addAttribute( 'position', new Float32BufferAttribute( positions, 3 ) );
	this.addAttribute( 'uv', new Float32BufferAttribute( uvs, 2 ) );

};

LineSegmentsGeometry.prototype = Object.assign( Object.create( InstancedBufferGeometry.prototype ), {

	constructor: LineSegmentsGeometry,

	isLineSegmentsGeometry: true,

	applyMatrix: function ( matrix ) {

		var start = this.attributes.instanceStart;
		var end = this.attributes.instanceEnd;

		if ( start !== undefined ) {

			matrix.applyToBufferAttribute( start );

			matrix.applyToBufferAttribute( end );

			start.data.needsUpdate = true;

		}

		if ( this.boundingBox !== null ) {

			this.computeBoundingBox();

		}

		if ( this.boundingSphere !== null ) {

			this.computeBoundingSphere();

		}

		return this;

	},

	setPositions: function ( array ) {

		var lineSegments;

		if ( array instanceof Float32Array ) {

			lineSegments = array;

		} else if ( Array.isArray( array ) ) {

			lineSegments = new Float32Array( array );

		}

		var instanceBuffer = new InstancedInterleavedBuffer( lineSegments, 6, 1 ); // xyz, xyz

		this.addAttribute( 'instanceStart', new InterleavedBufferAttribute( instanceBuffer, 3, 0 ) ); // xyz
		this.addAttribute( 'instanceEnd', new InterleavedBufferAttribute( instanceBuffer, 3, 3 ) ); // xyz

		//

		this.computeBoundingBox();
		this.computeBoundingSphere();

		return this;

	},

	setColors: function ( array ) {

		var colors;

		if ( array instanceof Float32Array ) {

			colors = array;

		} else if ( Array.isArray( array ) ) {

			colors = new Float32Array( array );

		}

		var instanceColorBuffer = new InstancedInterleavedBuffer( colors, 6, 1 ); // rgb, rgb

		this.addAttribute( 'instanceColorStart', new InterleavedBufferAttribute( instanceColorBuffer, 3, 0 ) ); // rgb
		this.addAttribute( 'instanceColorEnd', new InterleavedBufferAttribute( instanceColorBuffer, 3, 3 ) ); // rgb

		return this;

	},

	fromWireframeGeometry: function ( geometry ) {

		this.setPositions( geometry.attributes.position.array );

		return this;

	},

	fromEdgesGeometry: function ( geometry ) {

		this.setPositions( geometry.attributes.position.array );

		return this;

	},

	fromMesh: function ( mesh ) {

		this.fromWireframeGeometry( new WireframeGeometry( mesh.geometry ) );

		// set colors, maybe

		return this;

	},

	fromLineSegements: function ( lineSegments ) {

		var geometry = lineSegments.geometry;

		if ( geometry.isGeometry ) {

			this.setPositions( geometry.vertices );

		} else if ( geometry.isBufferGeometry ) {

			this.setPositions( geometry.position.array ); // assumes non-indexed

		}

		// set colors, maybe

		return this;

	},

	computeBoundingBox: function () {

		var box = new Box3();

		return function computeBoundingBox() {

			if ( this.boundingBox === null ) {

				this.boundingBox = new Box3();

			}

			var start = this.attributes.instanceStart;
			var end = this.attributes.instanceEnd;

			if ( start !== undefined && end !== undefined ) {

				this.boundingBox.setFromBufferAttribute( start );

				box.setFromBufferAttribute( end );

				this.boundingBox.union( box );

			}

		};

	}(),

	computeBoundingSphere: function () {

		var vector = new Vector3();

		return function computeBoundingSphere() {

			if ( this.boundingSphere === null ) {

				this.boundingSphere = new Sphere();

			}

			if ( this.boundingBox === null ) {

				this.computeBoundingBox();

			}

			var start = this.attributes.instanceStart;
			var end = this.attributes.instanceEnd;

			if ( start !== undefined && end !== undefined ) {

				var center = this.boundingSphere.center;

				this.boundingBox.getCenter( center );

				var maxRadiusSq = 0;

				for ( var i = 0, il = start.count; i < il; i ++ ) {

					vector.fromBufferAttribute( start, i );
					maxRadiusSq = Math.max( maxRadiusSq, center.distanceToSquared( vector ) );

					vector.fromBufferAttribute( end, i );
					maxRadiusSq = Math.max( maxRadiusSq, center.distanceToSquared( vector ) );

				}

				this.boundingSphere.radius = Math.sqrt( maxRadiusSq );

				if ( isNaN( this.boundingSphere.radius ) ) {

					console.error( 'THREE.LineSegmentsGeometry.computeBoundingSphere(): Computed radius is NaN. The instanced position data is likely to have NaN values.', this );

				}

			}

		};

	}(),

	toJSON: function () {

		// todo

	},

	clone: function () {

		// todo

	},

	copy: function ( /* source */ ) {

		// todo

		return this;

	}

} );

export { LineSegmentsGeometry };
