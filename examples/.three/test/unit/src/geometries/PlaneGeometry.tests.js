/**
 * @author TristanVALCKE / https://github.com/Itee
 * @author Anonymous
 */
/* global QUnit */

import { runStdGeometryTests } from '../../qunit-utils';
import {
	PlaneBufferGeometry
} from '../../../../src/geometries/PlaneGeometry';

export default QUnit.module( 'Geometries', () => {

	QUnit.module( 'PlaneBufferGeometry', ( hooks ) => {

		var geometries = undefined;
		hooks.beforeEach( function () {

			const parameters = {
				width: 10,
				height: 30,
				widthSegments: 3,
				heightSegments: 5
			};

			geometries = [
				new PlaneBufferGeometry(),
				new PlaneBufferGeometry( parameters.width ),
				new PlaneBufferGeometry( parameters.width, parameters.height ),
				new PlaneBufferGeometry( parameters.width, parameters.height, parameters.widthSegments ),
				new PlaneBufferGeometry( parameters.width, parameters.height, parameters.widthSegments, parameters.heightSegments ),
			];

		} );

		// INHERITANCE
		QUnit.todo( "Extending", ( assert ) => {

			assert.ok( false, "everything's gonna be alright" );

		} );

		// INSTANCING
		QUnit.todo( "Instancing", ( assert ) => {

			assert.ok( false, "everything's gonna be alright" );

		} );

		// OTHERS
		QUnit.test( 'Standard geometry tests', ( assert ) => {

			runStdGeometryTests( assert, geometries );

		} );

	} );

} );
