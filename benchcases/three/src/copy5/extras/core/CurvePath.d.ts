import { Curve } from './Curve';
import { Geometry } from './../../core/Geometry';
import { Vector } from './../../math/Vector2';

export class CurvePath<T extends Vector> extends Curve<T> {

	constructor();

	curves: Curve<T>[];
	autoClose: boolean;

	add( curve: Curve<T> ): void;
	checkConnection(): boolean;
	closePath(): void;
	getPoint( t: number ): T;
	getLength(): number;
	updateArcLengths(): void;
	getCurveLengths(): number[];
	getSpacedPoints( divisions?: number ): T[];
	getPoints( divisions?: number ): T[];

	/**
	 * @deprecated Use {@link Geometry#setFromPoints new THREE.Geometry().setFromPoints( points )} instead.
	 */
	createPointsGeometry( divisions: number ): Geometry;
	/**
	 * @deprecated Use {@link Geometry#setFromPoints new THREE.Geometry().setFromPoints( points )} instead.
	 */
	createSpacedPointsGeometry( divisions: number ): Geometry;
	/**
	 * @deprecated Use {@link Geometry#setFromPoints new THREE.Geometry().setFromPoints( points )} instead.
	 */
	createGeometry( points: T[] ): Geometry;

}
