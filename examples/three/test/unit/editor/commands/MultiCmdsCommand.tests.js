/**
 * @author TristanVALCKE / https://github.com/Itee
 */
/* global QUnit */

import { NothingsIsExportedYet } from '../../../../editor/js/commands/MultiCmdsCommand';

export default QUnit.module( 'Editor', () => {

	QUnit.module( 'Commands', () => {

		QUnit.module.todo( 'MultiCmdsCommand', () => {

			QUnit.test( 'write me !', ( assert ) => {

				assert.ok( false, "everything's gonna be alright" );

			} );

		} );

	} );

} );
