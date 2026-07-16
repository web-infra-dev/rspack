import type {
  JsRealContentHashPluginUpdateHashData,
  RegisterJsTaps,
} from './napi-binding';

declare const data: JsRealContentHashPluginUpdateHashData;

const assets: Buffer[] = data.assets;
const oldHash: string = data.oldHash;

declare const register: RegisterJsTaps['registerRealContentHashPluginUpdateHashTaps'];

register([])[0]?.function({ assets, oldHash });
