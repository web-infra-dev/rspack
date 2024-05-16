import { GlobalState } from "@/store";
import authentication, { AuthParams } from "@/utils/authentication";
import React, { useEffect, useState } from "react";
import { useSelector } from "react-redux";

type PermissionWrapperProps = AuthParams & {
	backup?: React.ReactNode;
};

const PermissionWrapper = (
	props: React.PropsWithChildren<PermissionWrapperProps>
) => {
	const { backup, requiredPermissions, oneOfPerm } = props;
	const [hasPermission, setHasPermission] = useState(false);
	const userInfo = useSelector((state: GlobalState) => state.userInfo);

	useEffect(() => {
		const hasPermission = authentication(
			{ requiredPermissions, oneOfPerm },
			userInfo.permissions
		);
		setHasPermission(hasPermission);
	}, [requiredPermissions, oneOfPerm, userInfo.permissions]);

	if (hasPermission) {
		return <>{convertReactElement(props.children)}</>;
	}
	if (backup) {
		return <>{convertReactElement(backup)}</>;
	}
	return null;
};

function convertReactElement(node: React.ReactNode): React.ReactElement {
	if (!React.isValidElement(node)) {
		return <>{node}</>;
	}
	return node;
}

export default PermissionWrapper;
