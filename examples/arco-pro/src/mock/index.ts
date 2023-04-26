import { isSSR } from "@/utils/is";
import Mock from "mockjs";

import "./message-box";
import "./user";

if (!isSSR) {
	Mock.setup({
		timeout: "500-1500"
	});
}
