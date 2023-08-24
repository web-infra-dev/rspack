import { BuiltinPluginKind, create } from "./base";

export const DevFriendlySplitChunksPlugin = create<undefined, undefined>(
	BuiltinPluginKind.DevFriendlySplitChunks,
	() => undefined
);
