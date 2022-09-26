/**
 * @author lxxxvi / https://github.com/lxxxvi
 * Developed as part of a project at University of Applied Sciences and Arts Northwestern Switzerland (www.fhnw.ch)
 */

QUnit.module( "AddObjectCommandAndRemoveObjectCommand" );

QUnit.test( "Test AddObjectCommand and RemoveObjectCommand (Undo and Redo)", function( assert ) {

	// setup
	var editor = new Editor();

	var box = aBox( 'The Box' );
	var light = aPointlight( 'The PointLight' );
	var camera = aPerspectiveCamera( 'The Camera' );

	var objects = [ box , light, camera ];

	objects.map( function( object ) {

		// Test Add
		var cmd = new AddObjectCommand( object );
		cmd.updatable = false;

		editor.execute( cmd );
		assert.ok( editor.scene.children.length == 1, "OK, adding '" + object.type + "' was successful " );

		editor.undo();
		assert.ok( editor.scene.children.length == 0, "OK, adding '" + object.type + "' is undone (was removed)" );

		editor.redo();
		assert.ok( editor.scene.children[ 0 ].name == object.name, "OK, removed '" + object.type + "' was added again (redo)" );

		assert.ok( editor.selected == object, "OK, focus was set on recovered object after Add-Redo" );


		// Test Remove
		var cmd = new RemoveObjectCommand( object );
		cmd.updatable = false;

		editor.execute( cmd );
		assert.ok( editor.scene.children.length == 0, "OK, removing object was successful" );

		editor.undo();
		assert.ok( editor.scene.children[ 0 ].name == object.name, "OK, removed object was added again (undo)" );

		assert.ok( editor.selected == object, "OK, focus was set on recovered object after Delete-Undo" );

		editor.redo();
		assert.ok( editor.scene.children.length == 0, "OK, object was removed again (redo)" );


	} );


} );
