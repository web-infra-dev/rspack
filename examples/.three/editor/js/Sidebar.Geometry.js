/**
 * @author mrdoob / http://mrdoob.com/
 */

Sidebar.Geometry = function ( editor ) {

	var strings = editor.strings;

	var signals = editor.signals;

	var container = new UI.Panel();
	container.setBorderTop( '0' );
	container.setDisplay( 'none' );
	container.setPaddingTop( '20px' );

	// Actions

	/*
	var objectActions = new UI.Select().setPosition( 'absolute' ).setRight( '8px' ).setFontSize( '11px' );
	objectActions.setOptions( {

		'Actions': 'Actions',
		'Center': 'Center',
		'Convert': 'Convert',
		'Flatten': 'Flatten'

	} );
	objectActions.onClick( function ( event ) {

		event.stopPropagation(); // Avoid panel collapsing

	} );
	objectActions.onChange( function ( event ) {

		var action = this.getValue();

		var object = editor.selected;
		var geometry = object.geometry;

		if ( confirm( action + ' ' + object.name + '?' ) === false ) return;

		switch ( action ) {

			case 'Center':

				var offset = geometry.center();

				var newPosition = object.position.clone();
				newPosition.sub( offset );
				editor.execute( new SetPositionCommand( editor, object, newPosition ) );

				editor.signals.geometryChanged.dispatch( object );

				break;

			case 'Convert':

				if ( geometry && geometry.isGeometry ) {

					editor.execute( new SetGeometryCommand( editor, object, new THREE.BufferGeometry().fromGeometry( geometry ) ) );

				}

				break;

			case 'Flatten':

				var newGeometry = geometry.clone();
				newGeometry.uuid = geometry.uuid;
				newGeometry.applyMatrix( object.matrix );

				var cmds = [ new SetGeometryCommand( editor, object, newGeometry ),
					new SetPositionCommand( editor, object, new THREE.Vector3( 0, 0, 0 ) ),
					new SetRotationCommand( editor, object, new THREE.Euler( 0, 0, 0 ) ),
					new SetScaleCommand( editor, object, new THREE.Vector3( 1, 1, 1 ) ) ];

				editor.execute( new MultiCmdsCommand( editor, cmds ), 'Flatten Geometry' );

				break;

		}

		this.setValue( 'Actions' );

	} );
	container.addStatic( objectActions );
	*/

	// type

	var geometryTypeRow = new UI.Row();
	var geometryType = new UI.Text();

	geometryTypeRow.add( new UI.Text( strings.getKey( 'sidebar/geometry/type' ) ).setWidth( '90px' ) );
	geometryTypeRow.add( geometryType );

	container.add( geometryTypeRow );

	// uuid

	var geometryUUIDRow = new UI.Row();
	var geometryUUID = new UI.Input().setWidth( '102px' ).setFontSize( '12px' ).setDisabled( true );
	var geometryUUIDRenew = new UI.Button( strings.getKey( 'sidebar/geometry/new' ) ).setMarginLeft( '7px' ).onClick( function () {

		geometryUUID.setValue( THREE.Math.generateUUID() );

		editor.execute( new SetGeometryValueCommand( editor, editor.selected, 'uuid', geometryUUID.getValue() ) );

	} );

	geometryUUIDRow.add( new UI.Text( strings.getKey( 'sidebar/geometry/uuid' ) ).setWidth( '90px' ) );
	geometryUUIDRow.add( geometryUUID );
	geometryUUIDRow.add( geometryUUIDRenew );

	container.add( geometryUUIDRow );

	// name

	var geometryNameRow = new UI.Row();
	var geometryName = new UI.Input().setWidth( '150px' ).setFontSize( '12px' ).onChange( function () {

		editor.execute( new SetGeometryValueCommand( editor, editor.selected, 'name', geometryName.getValue() ) );

	} );

	geometryNameRow.add( new UI.Text( strings.getKey( 'sidebar/geometry/name' ) ).setWidth( '90px' ) );
	geometryNameRow.add( geometryName );

	container.add( geometryNameRow );

	// parameters

	var parameters = new UI.Span();
	container.add( parameters );

	// geometry

	container.add( new Sidebar.Geometry.Geometry( editor ) );

	// buffergeometry

	container.add( new Sidebar.Geometry.BufferGeometry( editor ) );

	// size

	var geometryBoundingSphere = new UI.Text();

	container.add( new UI.Text( strings.getKey( 'sidebar/geometry/bounds' ) ).setWidth( '90px' ) );
	container.add( geometryBoundingSphere );

	//

	function build() {

		var object = editor.selected;

		if ( object && object.geometry ) {

			var geometry = object.geometry;

			container.setDisplay( 'block' );

			geometryType.setValue( geometry.type );

			geometryUUID.setValue( geometry.uuid );
			geometryName.setValue( geometry.name );

			//

			parameters.clear();

			if ( geometry.type === 'BufferGeometry' || geometry.type === 'Geometry' ) {

				parameters.add( new Sidebar.Geometry.Modifiers( editor, object ) );

			} else if ( Sidebar.Geometry[ geometry.type ] !== undefined ) {

				parameters.add( new Sidebar.Geometry[ geometry.type ]( editor, object ) );

			}

			if ( geometry.boundingSphere === null ) geometry.computeBoundingSphere();

			geometryBoundingSphere.setValue( Math.floor( geometry.boundingSphere.radius * 1000 ) / 1000 );

		} else {

			container.setDisplay( 'none' );

		}

	}

	signals.objectSelected.add( build );
	signals.geometryChanged.add( build );

	return container;

};
