// @ts-ignore
import { styled } from "@xstyled/styled-components";
import React from "react";

export const Button = () => {
	return (
		<Container>
			{/* @ts-ignore */}
			<div css={{ color: "red" }}>Hello, Rspack!</div>
		</Container>
	);
};

export const Container = styled.div`
	padding: 2rem;
	background-color: pink;
`;
