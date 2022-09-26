/**
 * @author bhouston / http://exocortex.com
 * @author TristanVALCKE / https://github.com/Itee
 */
/* global QUnit */

import { Box2 } from '../../../../src/math/Box2';
import { Vector2 } from '../../../../src/math/Vector2';
import {
	negInf2,
	posInf2,
	negOne2,
	zero2,
	one2,
	two2
} from './Constants.tests';

export default QUnit.module( 'Maths', () => {

	QUnit.module( 'Box2', () => {

		// INSTANCING
		QUnit.test( "Instancing", ( assert ) => {

			var a = new Box2();
			assert.ok( a.min.equals( posInf2 ), "Passed!" );
			assert.ok( a.max.equals( negInf2 ), "Passed!" );

			var a = new Box2( zero2.clone(), zero2.clone() );
			assert.ok( a.min.equals( zero2 ), "Passed!" );
			assert.ok( a.max.equals( zero2 ), "Passed!" );

			var a = new Box2( zero2.clone(), one2.clone() );
			assert.ok( a.min.equals( zero2 ), "Passed!" );
			assert.ok( a.max.equals( one2 ), "Passed!" );

		} );

		// PUBLIC STUFF
		QUnit.test( "set", ( assert ) => {

			var a = new Box2();

			a.set( zero2, one2 );
			assert.ok( a.min.equals( zero2 ), "Passed!" );
			assert.ok( a.max.equals( one2 ), "Passed!" );

		} );

		QUnit.test( "setFromPoints", ( assert ) => {

			var a = new Box2();

			a.setFromPoints( [ zero2, one2, two2 ] );
			assert.ok( a.min.equals( zero2 ), "Passed!" );
			assert.ok( a.max.equals( two2 ), "Passed!" );

			a.setFromPoints( [ one2 ] );
			assert.ok( a.min.equals( one2 ), "Passed!" );
			assert.ok( a.max.equals( one2 ), "Passed!" );

			a.setFromPoints( [] );
			assert.ok( a.isEmpty(), "Passed!" );

		} );

		QUnit.test( "setFromCenterAndSize", ( assert ) => {

			var a = new Box2();

			a.setFromCenterAndSize( zero2, two2 );
			assert.ok( a.min.equals( negOne2 ), "Passed!" );
			assert.ok( a.max.equals( one2 ), "Passed!" );

			a.setFromCenterAndSize( one2, two2 );
			assert.ok( a.min.equals( zero2 ), "Passed!" );
			assert.ok( a.max.equals( two2 ), "Passed!" );

			a.setFromCenterAndSize( zero2, zero2 );
			assert.ok( a.min.equals( zero2 ), "Passed!" );
			assert.ok( a.max.equals( zero2 ), "Passed!" );

		} );

		QUnit.test( "clone", ( assert ) => {


			var a = new Box2( zero2, zero2 );

			var b = a.clone();
			assert.ok( b.min.equals( zero2 ), "Passed!" );
			assert.ok( b.max.equals( zero2 ), "Passed!" );

			a = new Box2();
			var b = a.clone();
			assert.ok( b.min.equals( posInf2 ), "Passed!" );
			assert.ok( b.max.equals( negInf2 ), "Passed!" );

		} );

		QUnit.test( "copy", ( assert ) => {

			var a = new Box2( zero2.clone(), one2.clone() );
			var b = new Box2().copy( a );
			assert.ok( b.min.equals( zero2 ), "Passed!" );
			assert.ok( b.max.equals( one2 ), "Passed!" );

			// ensure that it is a true copy
			a.min = zero2;
			a.max = one2;
			assert.ok( b.min.equals( zero2 ), "Passed!" );
			assert.ok( b.max.equals( one2 ), "Passed!" );

		} );

		QUnit.test( "empty/makeEmpty", ( assert ) => {

			var a = new Box2();

			assert.ok( a.isEmpty(), "Passed!" );

			var a = new Box2( zero2.clone(), one2.clone() );
			assert.ok( ! a.isEmpty(), "Passed!" );

			a.makeEmpty();
			assert.ok( a.isEmpty(), "Passed!" );

		} );

		QUnit.test( "isEmpty", ( assert ) => {

			var a = new Box2( zero2.clone(), zero2.clone() );
			assert.ok( ! a.isEmpty(), "Passed!" );

			var a = new Box2( zero2.clone(), one2.clone() );
			assert.ok( ! a.isEmpty(), "Passed!" );

			var a = new Box2( two2.clone(), one2.clone() );
			assert.ok( a.isEmpty(), "Passed!" );

			var a = new Box2( posInf2.clone(), negInf2.clone() );
			assert.ok( a.isEmpty(), "Passed!" );

		} );

		QUnit.test( "getCenter", ( assert ) => {

			var a = new Box2( zero2.clone(), zero2.clone() );
			var center = new Vector2();
			assert.ok( a.getCenter( center ).equals( zero2 ), "Passed!" );

			var a = new Box2( zero2, one2 );
			var midpoint = one2.clone().multiplyScalar( 0.5 );
			assert.ok( a.getCenter( center ).equals( midpoint ), "Passed!" );

		} );

		QUnit.test( "getSize", ( assert ) => {

			var a = new Box2( zero2.clone(), zero2.clone() );
			var size = new Vector2();

			assert.ok( a.getSize( size ).equals( zero2 ), "Passed!" );

			var a = new Box2( zero2.clone(), one2.clone() );
			assert.ok( a.getSize( size ).equals( one2 ), "Passed!" );

		} );

		QUnit.test( "expandByPoint", ( assert ) => {

			var a = new Box2( zero2.clone(), zero2.clone() );
			var size = new Vector2();
			var center = new Vector2();

			a.expandByPoint( zero2 );
			assert.ok( a.getSize( size ).equals( zero2 ), "Passed!" );

			a.expandByPoint( one2 );
			assert.ok( a.getSize( size ).equals( one2 ), "Passed!" );

			a.expandByPoint( one2.clone().negate() );
			assert.ok( a.getSize( size ).equals( one2.clone().multiplyScalar( 2 ) ), "Passed!" );
			assert.ok( a.getCenter( center ).equals( zero2 ), "Passed!" );

		} );

		QUnit.test( "expandByVector", ( assert ) => {

			var a = new Box2( zero2.clone(), zero2.clone() );
			var size = new Vector2();
			var center = new Vector2();

			a.expandByVector( zero2 );
			assert.ok( a.getSize( size ).equals( zero2 ), "Passed!" );

			a.expandByVector( one2 );
			assert.ok( a.getSize( size ).equals( one2.clone().multiplyScalar( 2 ) ), "Passed!" );
			assert.ok( a.getCenter( center ).equals( zero2 ), "Passed!" );

		} );

		QUnit.test( "expandByScalar", ( assert ) => {

			var a = new Box2( zero2.clone(), zero2.clone() );
			var size = new Vector2();
			var center = new Vector2();

			a.expandByScalar( 0 );
			assert.ok( a.getSize( size ).equals( zero2 ), "Passed!" );

			a.expandByScalar( 1 );
			assert.ok( a.getSize( size ).equals( one2.clone().multiplyScalar( 2 ) ), "Passed!" );
			assert.ok( a.getCenter( center ).equals( zero2 ), "Passed!" );

		} );

		QUnit.test( "containsPoint", ( assert ) => {

			var a = new Box2( zero2.clone(), zero2.clone() );

			assert.ok( a.containsPoint( zero2 ), "Passed!" );
			assert.ok( ! a.containsPoint( one2 ), "Passed!" );

			a.expandByScalar( 1 );
			assert.ok( a.containsPoint( zero2 ), "Passed!" );
			assert.ok( a.containsPoint( one2 ), "Passed!" );
			assert.ok( a.containsPoint( one2.clone().negate() ), "Passed!" );

		} );

		QUnit.test( "containsBox", ( assert ) => {

			var a = new Box2( zero2.clone(), zero2.clone() );
			var b = new Box2( zero2.clone(), one2.clone() );
			var c = new Box2( one2.clone().negate(), one2.clone() );

			assert.ok( a.containsBox( a ), "Passed!" );
			assert.ok( ! a.containsBox( b ), "Passed!" );
			assert.ok( ! a.containsBox( c ), "Passed!" );

			assert.ok( b.containsBox( a ), "Passed!" );
			assert.ok( c.containsBox( a ), "Passed!" );
			assert.ok( ! b.containsBox( c ), "Passed!" );

		} );

		QUnit.test( "getParameter", ( assert ) => {

			var a = new Box2( zero2.clone(), one2.clone() );
			var b = new Box2( one2.clone().negate(), one2.clone() );

			var parameter = new Vector2();

			a.getParameter( zero2, parameter );
			assert.ok( parameter.equals( zero2 ), "Passed!" );
			a.getParameter( one2, parameter );
			assert.ok( parameter.equals( one2 ), "Passed!" );

			b.getParameter( one2.clone().negate(), parameter );
			assert.ok( parameter.equals( zero2 ), "Passed!" );
			b.getParameter( zero2, parameter );
			assert.ok( parameter.equals( new Vector2( 0.5, 0.5 ) ), "Passed!" );
			b.getParameter( one2, parameter );
			assert.ok( parameter.equals( one2 ), "Passed!" );

		} );

		QUnit.test( "intersectsBox", ( assert ) => {

			var a = new Box2( zero2.clone(), zero2.clone() );
			var b = new Box2( zero2.clone(), one2.clone() );
			var c = new Box2( one2.clone().negate(), one2.clone() );

			assert.ok( a.intersectsBox( a ), "Passed!" );
			assert.ok( a.intersectsBox( b ), "Passed!" );
			assert.ok( a.intersectsBox( c ), "Passed!" );

			assert.ok( b.intersectsBox( a ), "Passed!" );
			assert.ok( c.intersectsBox( a ), "Passed!" );
			assert.ok( b.intersectsBox( c ), "Passed!" );

			b.translate( two2 );
			assert.ok( ! a.intersectsBox( b ), "Passed!" );
			assert.ok( ! b.intersectsBox( a ), "Passed!" );
			assert.ok( ! b.intersectsBox( c ), "Passed!" );

		} );

		QUnit.test( "clampPoint", ( assert ) => {

			var a = new Box2( zero2.clone(), zero2.clone() );
			var b = new Box2( one2.clone().negate(), one2.clone() );

			var point = new Vector2();

			a.clampPoint( zero2, point );
			assert.ok( point.equals( new Vector2( 0, 0 ) ), "Passed!" );
			a.clampPoint( one2, point );
			assert.ok( point.equals( new Vector2( 0, 0 ) ), "Passed!" );
			a.clampPoint( one2.clone().negate(), point );
			assert.ok( point.equals( new Vector2( 0, 0 ) ), "Passed!" );

			b.clampPoint( two2, point );
			assert.ok( point.equals( new Vector2( 1, 1 ) ), "Passed!" );
			b.clampPoint( one2, point );
			assert.ok( point.equals( new Vector2( 1, 1 ) ), "Passed!" );
			b.clampPoint( zero2, point );
			assert.ok( point.equals( new Vector2( 0, 0 ) ), "Passed!" );
			b.clampPoint( one2.clone().negate(), point );
			assert.ok( point.equals( new Vector2( - 1, - 1 ) ), "Passed!" );
			b.clampPoint( two2.clone().negate(), point );
			assert.ok( point.equals( new Vector2( - 1, - 1 ) ), "Passed!" );

		} );

		QUnit.test( "distanceToPoint", ( assert ) => {

			var a = new Box2( zero2.clone(), zero2.clone() );
			var b = new Box2( one2.clone().negate(), one2.clone() );

			assert.ok( a.distanceToPoint( new Vector2( 0, 0 ) ) == 0, "Passed!" );
			assert.ok( a.distanceToPoint( new Vector2( 1, 1 ) ) == Math.sqrt( 2 ), "Passed!" );
			assert.ok( a.distanceToPoint( new Vector2( - 1, - 1 ) ) == Math.sqrt( 2 ), "Passed!" );

			assert.ok( b.distanceToPoint( new Vector2( 2, 2 ) ) == Math.sqrt( 2 ), "Passed!" );
			assert.ok( b.distanceToPoint( new Vector2( 1, 1 ) ) == 0, "Passed!" );
			assert.ok( b.distanceToPoint( new Vector2( 0, 0 ) ) == 0, "Passed!" );
			assert.ok( b.distanceToPoint( new Vector2( - 1, - 1 ) ) == 0, "Passed!" );
			assert.ok( b.distanceToPoint( new Vector2( - 2, - 2 ) ) == Math.sqrt( 2 ), "Passed!" );

		} );

		QUnit.test( "intersect", ( assert ) => {

			var a = new Box2( zero2.clone(), zero2.clone() );
			var b = new Box2( zero2.clone(), one2.clone() );
			var c = new Box2( one2.clone().negate(), one2.clone() );

			assert.ok( a.clone().intersect( a ).equals( a ), "Passed!" );
			assert.ok( a.clone().intersect( b ).equals( a ), "Passed!" );
			assert.ok( b.clone().intersect( b ).equals( b ), "Passed!" );
			assert.ok( a.clone().intersect( c ).equals( a ), "Passed!" );
			assert.ok( b.clone().intersect( c ).equals( b ), "Passed!" );
			assert.ok( c.clone().intersect( c ).equals( c ), "Passed!" );

		} );

		QUnit.test( "union", ( assert ) => {

			var a = new Box2( zero2.clone(), zero2.clone() );
			var b = new Box2( zero2.clone(), one2.clone() );
			var c = new Box2( one2.clone().negate(), one2.clone() );

			assert.ok( a.clone().union( a ).equals( a ), "Passed!" );
			assert.ok( a.clone().union( b ).equals( b ), "Passed!" );
			assert.ok( a.clone().union( c ).equals( c ), "Passed!" );
			assert.ok( b.clone().union( c ).equals( c ), "Passed!" );

		} );

		QUnit.test( "translate", ( assert ) => {

			var a = new Box2( zero2.clone(), zero2.clone() );
			var b = new Box2( zero2.clone(), one2.clone() );
			var c = new Box2( one2.clone().negate(), one2.clone() );
			var d = new Box2( one2.clone().negate(), zero2.clone() );

			assert.ok( a.clone().translate( one2 ).equals( new Box2( one2, one2 ) ), "Passed!" );
			assert.ok( a.clone().translate( one2 ).translate( one2.clone().negate() ).equals( a ), "Passed!" );
			assert.ok( d.clone().translate( one2 ).equals( b ), "Passed!" );
			assert.ok( b.clone().translate( one2.clone().negate() ).equals( d ), "Passed!" );

		} );

		QUnit.test( "equals", ( assert ) => {


			var a = new Box2();
			var b = new Box2();
			assert.ok( b.equals( a ), "Passed!" );
			assert.ok( a.equals( b ), "Passed!" );

			a = new Box2( one2, two2 );
			b = new Box2( one2, two2 );
			assert.ok( b.equals( a ), "Passed!" );
			assert.ok( a.equals( b ), "Passed!" );

			a = new Box2( one2, two2 );
			b = a.clone();
			assert.ok( b.equals( a ), "Passed!" );
			assert.ok( a.equals( b ), "Passed!" );

			a = new Box2( one2, two2 );
			b = new Box2( one2, one2 );
			assert.ok( ! b.equals( a ), "Passed!" );
			assert.ok( ! a.equals( b ), "Passed!" );

			a = new Box2();
			b = new Box2( one2, one2 );
			assert.ok( ! b.equals( a ), "Passed!" );
			assert.ok( ! a.equals( b ), "Passed!" );

			a = new Box2( one2, two2 );
			b = new Box2( one2, one2 );
			assert.ok( ! b.equals( a ), "Passed!" );
			assert.ok( ! a.equals( b ), "Passed!" );

		} );

	} );

} );
