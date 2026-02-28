import * as ENUM from "./enum.js";

function Record() {}

export const code2CreateChatDocPermission = {
	1: ENUM.a.a
};

export function getDocPermissionTextSendMe() {}

export class Doc extends Record({}) {
	isSheet() {
		return this.type === ENUM.b.b;
	}
}

Doc.fromJS = data => new Doc(data);
