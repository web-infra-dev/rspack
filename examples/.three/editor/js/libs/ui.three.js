/**
 * @author mrdoob / http://mrdoob.com/
 */

UI.Texture = function ( mapping ) {

	UI.Element.call( this );

	var scope = this;

	var dom = document.createElement( 'span' );

	var form = document.createElement( 'form' );

	var input = document.createElement( 'input' );
	input.type = 'file';
	input.addEventListener( 'change', function ( event ) {

		loadFile( event.target.files[ 0 ] );

	} );
	form.appendChild( input );

	var canvas = document.createElement( 'canvas' );
	canvas.width = 32;
	canvas.height = 16;
	canvas.style.cursor = 'pointer';
	canvas.style.marginRight = '5px';
	canvas.style.border = '1px solid #888';
	canvas.addEventListener( 'click', function ( event ) {

		input.click();

	}, false );
	canvas.addEventListener( 'drop', function ( event ) {

		event.preventDefault();
		event.stopPropagation();
		loadFile( event.dataTransfer.files[ 0 ] );

	}, false );
	dom.appendChild( canvas );

	function loadFile( file ) {

		if ( file.type.match( 'image.*' ) ) {

			var reader = new FileReader();

			if ( file.type === 'image/targa' ) {

				reader.addEventListener( 'load', function ( event ) {

					var canvas = new THREE.TGALoader().parse( event.target.result );

					var texture = new THREE.CanvasTexture( canvas, mapping );
					texture.sourceFile = file.name;

					scope.setValue( texture );

					if ( scope.onChangeCallback ) scope.onChangeCallback( texture );

				}, false );

				reader.readAsArrayBuffer( file );

			} else {

				reader.addEventListener( 'load', function ( event ) {

					var image = document.createElement( 'img' );
					image.addEventListener( 'load', function ( event ) {

						var texture = new THREE.Texture( this, mapping );
						texture.sourceFile = file.name;
						texture.format = file.type === 'image/jpeg' ? THREE.RGBFormat : THREE.RGBAFormat;
						texture.needsUpdate = true;

						scope.setValue( texture );

						if ( scope.onChangeCallback ) scope.onChangeCallback( texture );

					}, false );

					image.src = event.target.result;

				}, false );

				reader.readAsDataURL( file );

			}

		}

		form.reset();

	}

	this.dom = dom;
	this.texture = null;
	this.onChangeCallback = null;

	return this;

};

UI.Texture.prototype = Object.create( UI.Element.prototype );
UI.Texture.prototype.constructor = UI.Texture;

UI.Texture.prototype.getValue = function () {

	return this.texture;

};

UI.Texture.prototype.setValue = function ( texture ) {

	var canvas = this.dom.children[ 0 ];
	var context = canvas.getContext( '2d' );

	if ( texture !== null ) {

		var image = texture.image;

		if ( image !== undefined && image.width > 0 ) {

			canvas.title = texture.sourceFile;

			var scale = canvas.width / image.width;
			context.drawImage( image, 0, 0, image.width * scale, image.height * scale );

		} else {

			canvas.title = texture.sourceFile + ' (error)';
			context.clearRect( 0, 0, canvas.width, canvas.height );

		}

	} else {

		canvas.title = 'empty';

		if ( context !== null ) {

			// Seems like context can be null if the canvas is not visible

			context.clearRect( 0, 0, canvas.width, canvas.height );

		}

	}

	this.texture = texture;

};

UI.Texture.prototype.setEncoding = function ( encoding ) {

	var texture = this.getValue();
	if ( texture !== null ) {

		texture.encoding = encoding;

	}

	return this;

};

UI.Texture.prototype.onChange = function ( callback ) {

	this.onChangeCallback = callback;

	return this;

};

// Outliner

UI.Outliner = function ( editor ) {

	UI.Element.call( this );

	var scope = this;

	var dom = document.createElement( 'div' );
	dom.className = 'Outliner';
	dom.tabIndex = 0;	// keyup event is ignored without setting tabIndex

	// hack
	this.scene = editor.scene;

	// Prevent native scroll behavior
	dom.addEventListener( 'keydown', function ( event ) {

		switch ( event.keyCode ) {

			case 38: // up
			case 40: // down
				event.preventDefault();
				event.stopPropagation();
				break;

		}

	}, false );

	// Keybindings to support arrow navigation
	dom.addEventListener( 'keyup', function ( event ) {

		switch ( event.keyCode ) {

			case 38: // up
				scope.selectIndex( scope.selectedIndex - 1 );
				break;
			case 40: // down
				scope.selectIndex( scope.selectedIndex + 1 );
				break;

		}

	}, false );

	this.dom = dom;

	this.options = [];
	this.selectedIndex = - 1;
	this.selectedValue = null;

	return this;

};

UI.Outliner.prototype = Object.create( UI.Element.prototype );
UI.Outliner.prototype.constructor = UI.Outliner;

UI.Outliner.prototype.selectIndex = function ( index ) {

	if ( index >= 0 && index < this.options.length ) {

		this.setValue( this.options[ index ].value );

		var changeEvent = document.createEvent( 'HTMLEvents' );
		changeEvent.initEvent( 'change', true, true );
		this.dom.dispatchEvent( changeEvent );

	}

};

UI.Outliner.prototype.setOptions = function ( options ) {

	var scope = this;

	while ( scope.dom.children.length > 0 ) {

		scope.dom.removeChild( scope.dom.firstChild );

	}

	function onClick() {

		scope.setValue( this.value );

		var changeEvent = document.createEvent( 'HTMLEvents' );
		changeEvent.initEvent( 'change', true, true );
		scope.dom.dispatchEvent( changeEvent );

	}

	// Drag

	var currentDrag;

	function onDrag( event ) {

		currentDrag = this;

	}

	function onDragStart( event ) {

		event.dataTransfer.setData( 'text', 'foo' );

	}

	function onDragOver( event ) {

		if ( this === currentDrag ) return;

		var area = event.offsetY / this.clientHeight;

		if ( area < 0.25 ) {

			this.className = 'option dragTop';

		} else if ( area > 0.75 ) {

			this.className = 'option dragBottom';

		} else {

			this.className = 'option drag';

		}

	}

	function onDragLeave() {

		if ( this === currentDrag ) return;

		this.className = 'option';

	}

	function onDrop( event ) {

		if ( this === currentDrag ) return;

		this.className = 'option';

		var scene = scope.scene;
		var object = scene.getObjectById( currentDrag.value );

		var area = event.offsetY / this.clientHeight;

		if ( area < 0.25 ) {

			var nextObject = scene.getObjectById( this.value );
			moveObject( object, nextObject.parent, nextObject );

		} else if ( area > 0.75 ) {

			var nextObject = scene.getObjectById( this.nextSibling.value );
			moveObject( object, nextObject.parent, nextObject );

		} else {

			var parentObject = scene.getObjectById( this.value );
			moveObject( object, parentObject );

		}

	}

	function moveObject( object, newParent, nextObject ) {

		if ( nextObject === null ) nextObject = undefined;

		var newParentIsChild = false;

		object.traverse( function ( child ) {

			if ( child === newParent ) newParentIsChild = true;

		} );

		if ( newParentIsChild ) return;

		editor.execute( new MoveObjectCommand( editor, object, newParent, nextObject ) );

		var changeEvent = document.createEvent( 'HTMLEvents' );
		changeEvent.initEvent( 'change', true, true );
		scope.dom.dispatchEvent( changeEvent );

	}

	//

	scope.options = [];

	for ( var i = 0; i < options.length; i ++ ) {

		var div = options[ i ];
		div.className = 'option';
		scope.dom.appendChild( div );

		scope.options.push( div );

		div.addEventListener( 'click', onClick, false );

		if ( div.draggable === true ) {

			div.addEventListener( 'drag', onDrag, false );
			div.addEventListener( 'dragstart', onDragStart, false ); // Firefox needs this

			div.addEventListener( 'dragover', onDragOver, false );
			div.addEventListener( 'dragleave', onDragLeave, false );
			div.addEventListener( 'drop', onDrop, false );

		}


	}

	return scope;

};

UI.Outliner.prototype.getValue = function () {

	return this.selectedValue;

};

UI.Outliner.prototype.setValue = function ( value ) {

	for ( var i = 0; i < this.options.length; i ++ ) {

		var element = this.options[ i ];

		if ( element.value === value ) {

			element.classList.add( 'active' );

			// scroll into view

			var y = element.offsetTop - this.dom.offsetTop;
			var bottomY = y + element.offsetHeight;
			var minScroll = bottomY - this.dom.offsetHeight;

			if ( this.dom.scrollTop > y ) {

				this.dom.scrollTop = y;

			} else if ( this.dom.scrollTop < minScroll ) {

				this.dom.scrollTop = minScroll;

			}

			this.selectedIndex = i;

		} else {

			element.classList.remove( 'active' );

		}

	}

	this.selectedValue = value;

	return this;

};

var Points = function ( onAddClicked ) {

	UI.Element.call( this );

	var span = new UI.Span().setDisplay( 'inline-block' );

	this.pointsList = new UI.Div();
	span.add( this.pointsList );

	var row = new UI.Row();
	span.add( row );

	var addPointButton = new UI.Button( '+' ).onClick( onAddClicked );
	row.add( addPointButton );

	this.update = function () {

		if ( this.onChangeCallback !== null ) {

			this.onChangeCallback();

		}

	}.bind( this );

	this.dom = span.dom;
	this.pointsUI = [];
	this.lastPointIdx = 0;
	this.onChangeCallback = null;
	return this;

};

Points.prototype = Object.create( UI.Element.prototype );
Points.prototype.constructor = Points;

Points.prototype.onChange = function ( callback ) {

	this.onChangeCallback = callback;

	return this;

};

Points.prototype.clear = function () {

	for ( var i = 0; i < this.pointsUI.length; ++ i ) {

		if ( this.pointsUI[ i ] ) {

			this.deletePointRow( i, true );

		}

	}

	this.lastPointIdx = 0;

};

Points.prototype.deletePointRow = function ( idx, dontUpdate ) {

	if ( ! this.pointsUI[ idx ] ) return;

	this.pointsList.remove( this.pointsUI[ idx ].row );
	this.pointsUI[ idx ] = null;

	if ( dontUpdate !== true ) {

		this.update();

	}

};

UI.Points2 = function () {

	Points.call( this, UI.Points2.addRow.bind( this ) );

	return this;

};

UI.Points2.prototype = Object.create( Points.prototype );
UI.Points2.prototype.constructor = UI.Points2;

UI.Points2.addRow = function () {

	if ( this.pointsUI.length === 0 ) {

		this.pointsList.add( this.createPointRow( 0, 0 ) );

	} else {

		var point = this.pointsUI[ this.pointsUI.length - 1 ];

		this.pointsList.add( this.createPointRow( point.x.getValue(), point.y.getValue() ) );

	}

	this.update();

};

UI.Points2.prototype.getValue = function () {

	var points = [];
	var count = 0;

	for ( var i = 0; i < this.pointsUI.length; i ++ ) {

		var pointUI = this.pointsUI[ i ];

		if ( ! pointUI ) continue;

		points.push( new THREE.Vector2( pointUI.x.getValue(), pointUI.y.getValue() ) );
		++ count;
		pointUI.lbl.setValue( count );

	}

	return points;

};

UI.Points2.prototype.setValue = function ( points ) {

	this.clear();

	for ( var i = 0; i < points.length; i ++ ) {

		var point = points[ i ];
		this.pointsList.add( this.createPointRow( point.x, point.y ) );

	}

	this.update();
	return this;

};

UI.Points2.prototype.createPointRow = function ( x, y ) {

	var pointRow = new UI.Div();
	var lbl = new UI.Text( this.lastPointIdx + 1 ).setWidth( '20px' );
	var txtX = new UI.Number( x ).setWidth( '30px' ).onChange( this.update );
	var txtY = new UI.Number( y ).setWidth( '30px' ).onChange( this.update );

	var idx = this.lastPointIdx;
	var scope = this;
	var btn = new UI.Button( '-' ).onClick( function () {

		if ( scope.isEditing ) return;
		scope.deletePointRow( idx );

	} );

	this.pointsUI.push( { row: pointRow, lbl: lbl, x: txtX, y: txtY } );
	++ this.lastPointIdx;
	pointRow.add( lbl, txtX, txtY, btn );

	return pointRow;

};

UI.Points3 = function () {

	Points.call( this, UI.Points3.addRow.bind( this ) );

	return this;

};

UI.Points3.prototype = Object.create( Points.prototype );
UI.Points3.prototype.constructor = UI.Points3;

UI.Points3.addRow = function () {

	if ( this.pointsUI.length === 0 ) {

		this.pointsList.add( this.createPointRow( 0, 0, 0 ) );

	} else {

		var point = this.pointsUI[ this.pointsUI.length - 1 ];

		this.pointsList.add( this.createPointRow( point.x.getValue(), point.y.getValue(), point.z.getValue() ) );

	}

	this.update();

};

UI.Points3.prototype.getValue = function () {

	var points = [];
	var count = 0;

	for ( var i = 0; i < this.pointsUI.length; i ++ ) {

		var pointUI = this.pointsUI[ i ];

		if ( ! pointUI ) continue;

		points.push( new THREE.Vector3( pointUI.x.getValue(), pointUI.y.getValue(), pointUI.z.getValue() ) );
		++ count;
		pointUI.lbl.setValue( count );

	}

	return points;

};

UI.Points3.prototype.setValue = function ( points ) {

	this.clear();

	for ( var i = 0; i < points.length; i ++ ) {

		var point = points[ i ];
		this.pointsList.add( this.createPointRow( point.x, point.y, point.z ) );

	}

	this.update();
	return this;

};

UI.Points3.prototype.createPointRow = function ( x, y, z ) {

	var pointRow = new UI.Div();
	var lbl = new UI.Text( this.lastPointIdx + 1 ).setWidth( '20px' );
	var txtX = new UI.Number( x ).setWidth( '30px' ).onChange( this.update );
	var txtY = new UI.Number( y ).setWidth( '30px' ).onChange( this.update );
	var txtZ = new UI.Number( z ).setWidth( '30px' ).onChange( this.update );

	var idx = this.lastPointIdx;
	var scope = this;
	var btn = new UI.Button( '-' ).onClick( function () {

		if ( scope.isEditing ) return;
		scope.deletePointRow( idx );

	} );

	this.pointsUI.push( { row: pointRow, lbl: lbl, x: txtX, y: txtY, z: txtZ } );
	++ this.lastPointIdx;
	pointRow.add( lbl, txtX, txtY, txtZ, btn );

	return pointRow;

};

UI.THREE = {};

UI.THREE.Boolean = function ( boolean, text ) {

	UI.Span.call( this );

	this.setMarginRight( '10px' );

	this.checkbox = new UI.Checkbox( boolean );
	this.text = new UI.Text( text ).setMarginLeft( '3px' );

	this.add( this.checkbox );
	this.add( this.text );

};

UI.THREE.Boolean.prototype = Object.create( UI.Span.prototype );
UI.THREE.Boolean.prototype.constructor = UI.THREE.Boolean;

UI.THREE.Boolean.prototype.getValue = function () {

	return this.checkbox.getValue();

};

UI.THREE.Boolean.prototype.setValue = function ( value ) {

	return this.checkbox.setValue( value );

};
