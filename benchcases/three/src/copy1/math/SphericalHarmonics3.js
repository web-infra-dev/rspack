import { Vector3 } from './Vector3.js';

/**
 * @author bhouston / http://clara.io
 * @author WestLangley / http://github.com/WestLangley
 *
 * Primary reference:
 *   https://graphics.stanford.edu/papers/envmap/envmap.pdf
 *
 * Secondary reference:
 *   https://www.ppsloan.org/publications/StupidSH36.pdf
 */

// 3-band SH defined by 9 coefficients

function SphericalHarmonics3() {

	this.coefficients = [];

	for ( var i = 0; i < 9; i ++ ) {

		this.coefficients.push( new Vector3() );

	}

}

Object.assign( SphericalHarmonics3.prototype, {

	isSphericalHarmonics3: true,

	set: function ( coefficients ) {

		for ( var i = 0; i < 9; i ++ ) {

			this.coefficients[ i ].copy( coefficients[ i ] );

		}

		return this;

	},

	zero: function () {

		for ( var i = 0; i < 9; i ++ ) {

			this.coefficients[ i ].set( 0, 0, 0 );

		}

		return this;

	},

	// get the radiance in the direction of the normal
	// target is a Vector3
	getAt: function ( normal, target ) {

		// normal is assumed to be unit length

		var x = normal.x, y = normal.y, z = normal.z;

		var coeff = this.coefficients;

		// band 0
		target.copy( coeff[ 0 ] ).multiplyScalar( 0.282095 );

		// band 1
		target.addScale( coeff[ 1 ], 0.488603 * y );
		target.addScale( coeff[ 2 ], 0.488603 * z );
		target.addScale( coeff[ 3 ], 0.488603 * x );

		// band 2
		target.addScale( coeff[ 4 ], 1.092548 * ( x * y ) );
		target.addScale( coeff[ 5 ], 1.092548 * ( y * z ) );
		target.addScale( coeff[ 6 ], 0.315392 * ( 3.0 * z * z - 1.0 ) );
		target.addScale( coeff[ 7 ], 1.092548 * ( x * z ) );
		target.addScale( coeff[ 8 ], 0.546274 * ( x * x - y * y ) );

		return target;

	},

	// get the irradiance (radiance convolved with cosine lobe) in the direction of the normal
	// target is a Vector3
	// https://graphics.stanford.edu/papers/envmap/envmap.pdf
	getIrradianceAt: function ( normal, target ) {

		// normal is assumed to be unit length

		var x = normal.x, y = normal.y, z = normal.z;

		var coeff = this.coefficients;

		// band 0
		target.copy( coeff[ 0 ] ).multiplyScalar( 0.886227 ); // π * 0.282095

		// band 1
		target.addScale( coeff[ 1 ], 2.0 * 0.511664 * y ); // ( 2 * π / 3 ) * 0.488603
		target.addScale( coeff[ 2 ], 2.0 * 0.511664 * z );
		target.addScale( coeff[ 3 ], 2.0 * 0.511664 * x );

		// band 2
		target.addScale( coeff[ 4 ], 2.0 * 0.429043 * x * y ); // ( π / 4 ) * 1.092548
		target.addScale( coeff[ 5 ], 2.0 * 0.429043 * y * z );
		target.addScale( coeff[ 6 ], 0.743125 * z * z - 0.247708 ); // ( π / 4 ) * 0.315392 * 3
		target.addScale( coeff[ 7 ], 2.0 * 0.429043 * x * z );
		target.addScale( coeff[ 8 ], 0.429043 * ( x * x - y * y ) ); // ( π / 4 ) * 0.546274

		return target;

	},

	add: function ( sh ) {

		for ( var i = 0; i < 9; i ++ ) {

			this.coefficients[ i ].add( sh.coefficients[ i ] );

		}

		return this;

	},


	scale: function ( s ) {

		for ( var i = 0; i < 9; i ++ ) {

			this.coefficients[ i ].multiplyScalar( s );

		}

		return this;

	},

	lerp: function ( sh, alpha ) {

		for ( var i = 0; i < 9; i ++ ) {

			this.coefficients[ i ].lerp( sh.coefficients[ i ], alpha );

		}

		return this;

	},

	equals: function ( sh ) {

		for ( var i = 0; i < 9; i ++ ) {

			if ( ! this.coefficients[ i ].equals( sh.coefficients[ i ] ) ) {

				return false;

			}

		}

		return true;

	},

	copy: function ( sh ) {

		return this.set( sh.coefficients );

	},

	clone: function () {

		return new this.constructor().copy( this );

	},

	fromArray: function ( array, offset ) {

		if ( offset === undefined ) offset = 0;

		var coefficients = this.coefficients;

		for ( var i = 0; i < 9; i ++ ) {

			coefficients[ i ].fromArray( array, offset + ( i * 3 ) );

		}

		return this;

	},

	toArray: function ( array, offset ) {

		if ( array === undefined ) array = [];
		if ( offset === undefined ) offset = 0;

		var coefficients = this.coefficients;

		for ( var i = 0; i < 9; i ++ ) {

			coefficients[ i ].toArray( array, offset + ( i * 3 ) );

		}

		return array;

	}

} );

Object.assign( SphericalHarmonics3, {

	// evaluate the basis functions
	// shBasis is an Array[ 9 ]
	getBasisAt: function ( normal, shBasis ) {

		// normal is assumed to be unit length

		var x = normal.x, y = normal.y, z = normal.z;

		// band 0
		shBasis[ 0 ] = 0.282095;

		// band 1
		shBasis[ 1 ] = 0.488603 * y;
		shBasis[ 2 ] = 0.488603 * z;
		shBasis[ 3 ] = 0.488603 * x;

		// band 2
		shBasis[ 4 ] = 1.092548 * x * y;
		shBasis[ 5 ] = 1.092548 * y * z;
		shBasis[ 6 ] = 0.315392 * ( 3 * z * z - 1 );
		shBasis[ 7 ] = 1.092548 * x * z;
		shBasis[ 8 ] = 0.546274 * ( x * x - y * y );

	}

} );

export { SphericalHarmonics3 };
