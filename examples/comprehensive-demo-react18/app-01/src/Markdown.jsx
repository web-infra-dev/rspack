import { Box, Link, Paper, Typography, styled } from "@mui/material";
import React from "react";
import ReactMarkdown from "markdown-to-jsx";

const StyledListItem = styled("li")(({ theme }) => ({
	marginTop: theme.spacing(1)
}));

const StyledBlockquote = styled("blockquote")(({ theme }) => ({
	margin: 0,
	padding: theme.spacing(2, 0, 2, 4),
	borderLeft: `${theme.spacing(1)}px solid ${theme.palette.divider}`,
	color: theme.palette.text.secondary
}));

const options = {
	overrides: {
		h1: {
			component: Typography,
			props: {
				gutterBottom: true,
				variant: "h5"
			}
		},
		h2: { component: Typography, props: { gutterBottom: true, variant: "h6" } },
		h3: {
			component: Typography,
			props: { gutterBottom: true, variant: "subtitle1" }
		},
		h4: {
			component: Typography,
			props: { gutterBottom: true, variant: "caption", paragraph: true }
		},
		p: { component: Typography, props: { paragraph: true } },
		a: { component: Link },
		li: {
			component: ({ ...props }) => (
				<StyledListItem>
					<Typography component="span" {...props} />
				</StyledListItem>
			)
		},
		pre: {
			component: Paper,
			props: { elevation: 0, sx: { padding: "4px 8px" } }
		},
		blockquote: {
			component: ({ ...props }) => (
				<StyledBlockquote>
					<Typography component="span" {...props} />
				</StyledBlockquote>
			)
		}
	}
};

export default function Markdown(props) {
	return <ReactMarkdown options={options} {...props} />;
}
