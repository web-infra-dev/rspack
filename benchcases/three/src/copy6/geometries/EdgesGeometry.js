/**
 * @author WestLangley / http://github.com/WestLangley
 * @author Mugen87 / https://github.com/Mugen87
 */

import { BufferGeometry } from '../core/BufferGeometry.js';
import { Float32BufferAttribute } from '../core/BufferAttribute.js';
import { Geometry } from '../core/Geometry.js';
import { _Math } from '../math/Math.js';

function EdgesGeometry( geometry, thresholdAngle ) {

	BufferGeometry.call( this );

	this.type = 'EdgesGeometry';

	this.parameters = {
		thresholdAngle: thresholdAngle
	};

	thresholdAngle = ( thresholdAngle !== undefined ) ? thresholdAngle : 1;

	// buffer

	var vertices = [];

	// helper variables

	var thresholdDot = Math.cos( _Math.DEG2RAD * thresholdAngle );
	var edge = [ 0, 0 ], edges = {}, edge1, edge2;
	var key, keys = [ 'a', 'b', 'c' ];

	// prepare source geometry

	var geometry2;

	if ( geometry.isBufferGeometry ) {

		geometry2 = new Geometry();
		geometry2.fromBufferGeometry( geometry );

	} else {

		geometry2 = geometry.clone();

	}

	geometry2.mergeVertices();
	geometry2.computeFaceNormals();

	var sourceVertices = geometry2.vertices;
	var faces = geometry2.faces;

	// now create a data structure where each entry represents an edge with its adjoining faces

	for ( var i = 0, l = faces.length; i < l; i ++ ) {

		var face = faces[ i ];

		for ( var j = 0; j < 3; j ++ ) {

			edge1 = face[ keys[ j ] ];
			edge2 = face[ keys[ ( j + 1 ) % 3 ] ];
			edge[ 0 ] = Math.min( edge1, edge2 );
			edge[ 1 ] = Math.max( edge1, edge2 );

			key = edge[ 0 ] + ',' + edge[ 1 ];

			if ( edges[ key ] === undefined ) {

				edges[ key ] = { index1: edge[ 0 ], index2: edge[ 1 ], face1: i, face2: undefined };

			} else {

				edges[ key ].face2 = i;

			}

		}

	}

	// generate vertices

	for ( key in edges ) {

		var e = edges[ key ];

		// an edge is only rendered if the angle (in degrees) between the face normals of the adjoining faces exceeds this value. default = 1 degree.

		if ( e.face2 === undefined || faces[ e.face1 ].normal.dot( faces[ e.face2 ].normal ) <= thresholdDot ) {

			var vertex = sourceVertices[ e.index1 ];
			vertices.push( vertex.x, vertex.y, vertex.z );

			vertex = sourceVertices[ e.index2 ];
			vertices.push( vertex.x, vertex.y, vertex.z );

		}

	}

	// build geometry

	this.addAttribute( 'position', new Float32BufferAttribute( vertices, 3 ) );

}

EdgesGeometry.prototype = Object.create( BufferGeometry.prototype );
EdgesGeometry.prototype.constructor = EdgesGeometry;


export { EdgesGeometry };
