import { Sphere } from './../math/Sphere';
import { Geometry } from './../core/Geometry';
import { BufferGeometry } from './../core/BufferGeometry';

export class PolyhedronBufferGeometry extends BufferGeometry {

	constructor(
		vertices: number[],
		indices: number[],
		radius?: number,
		detail?: number
	);

	parameters: {
		vertices: number[];
		indices: number[];
		radius: number;
		detail: number;
	};

}

export class PolyhedronGeometry extends Geometry {

	constructor(
		vertices: number[],
		indices: number[],
		radius?: number,
		detail?: number
	);

	parameters: {
		vertices: number[];
		indices: number[];
		radius: number;
		detail: number;
	};
	boundingSphere: Sphere;

}
