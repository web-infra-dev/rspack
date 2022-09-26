import { Geometry } from './core/Geometry';
import { Material } from './materials/Material';
import { Object3D } from './core/Object3D';
import { Scene } from './scenes/Scene';

export namespace SceneUtils {
	export function createMultiMaterialObject(
		geometry: Geometry,
		materials: Material[]
	): Object3D;
	export function detach( child: Object3D, parent: Object3D, scene: Scene ): void;
	export function attach( child: Object3D, scene: Scene, parent: Object3D ): void;
}
