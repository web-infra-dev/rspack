import { Object3D } from './../core/Object3D';
import { Bone } from './../objects/Bone';
import { LineSegments } from './../objects/LineSegments';

export class SkeletonHelper extends LineSegments {

	constructor( bone: Object3D );

	bones: Bone[];
	root: Object3D;

	getBoneList( object: Object3D ): Bone[];
	update(): void;

}
