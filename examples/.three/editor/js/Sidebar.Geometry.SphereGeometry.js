/**
 * @author mrdoob / http://mrdoob.com/
 */

Sidebar.Geometry.SphereGeometry = function ( editor, object ) {

	var strings = editor.strings;

	var signals = editor.signals;

	var container = new UI.Row();

	var geometry = object.geometry;
	var parameters = geometry.parameters;

	// radius

	var radiusRow = new UI.Row();
	var radius = new UI.Number( parameters.radius ).onChange( update );

	radiusRow.add( new UI.Text( strings.getKey( 'sidebar/geometry/sphere_geometry/radius' ) ).setWidth( '90px' ) );
	radiusRow.add( radius );

	container.add( radiusRow );

	// widthSegments

	var widthSegmentsRow = new UI.Row();
	var widthSegments = new UI.Integer( parameters.widthSegments ).setRange( 1, Infinity ).onChange( update );

	widthSegmentsRow.add( new UI.Text( strings.getKey( 'sidebar/geometry/sphere_geometry/widthsegments' ) ).setWidth( '90px' ) );
	widthSegmentsRow.add( widthSegments );

	container.add( widthSegmentsRow );

	// heightSegments

	var heightSegmentsRow = new UI.Row();
	var heightSegments = new UI.Integer( parameters.heightSegments ).setRange( 1, Infinity ).onChange( update );

	heightSegmentsRow.add( new UI.Text( strings.getKey( 'sidebar/geometry/sphere_geometry/heightsegments' ) ).setWidth( '90px' ) );
	heightSegmentsRow.add( heightSegments );

	container.add( heightSegmentsRow );

	// phiStart

	var phiStartRow = new UI.Row();
	var phiStart = new UI.Number( parameters.phiStart * THREE.Math.RAD2DEG ).setStep( 10 ).onChange( update );

	phiStartRow.add( new UI.Text( strings.getKey( 'sidebar/geometry/sphere_geometry/phistart' ) ).setWidth( '90px' ) );
	phiStartRow.add( phiStart );

	container.add( phiStartRow );

	// phiLength

	var phiLengthRow = new UI.Row();
	var phiLength = new UI.Number( parameters.phiLength * THREE.Math.RAD2DEG ).setStep( 10 ).onChange( update );

	phiLengthRow.add( new UI.Text( strings.getKey( 'sidebar/geometry/sphere_geometry/philength' ) ).setWidth( '90px' ) );
	phiLengthRow.add( phiLength );

	container.add( phiLengthRow );

	// thetaStart

	var thetaStartRow = new UI.Row();
	var thetaStart = new UI.Number( parameters.thetaStart * THREE.Math.RAD2DEG ).setStep( 10 ).onChange( update );

	thetaStartRow.add( new UI.Text( strings.getKey( 'sidebar/geometry/sphere_geometry/thetastart' ) ).setWidth( '90px' ) );
	thetaStartRow.add( thetaStart );

	container.add( thetaStartRow );

	// thetaLength

	var thetaLengthRow = new UI.Row();
	var thetaLength = new UI.Number( parameters.thetaLength * THREE.Math.RAD2DEG ).setStep( 10 ).onChange( update );

	thetaLengthRow.add( new UI.Text( strings.getKey( 'sidebar/geometry/sphere_geometry/thetalength' ) ).setWidth( '90px' ) );
	thetaLengthRow.add( thetaLength );

	container.add( thetaLengthRow );


	//

	function update() {

		editor.execute( new SetGeometryCommand( editor, object, new THREE[ geometry.type ](
			radius.getValue(),
			widthSegments.getValue(),
			heightSegments.getValue(),
			phiStart.getValue() * THREE.Math.DEG2RAD,
			phiLength.getValue() * THREE.Math.DEG2RAD,
			thetaStart.getValue() * THREE.Math.DEG2RAD,
			thetaLength.getValue() * THREE.Math.DEG2RAD
		) ) );

	}

	return container;

};

Sidebar.Geometry.SphereBufferGeometry = Sidebar.Geometry.SphereGeometry;
