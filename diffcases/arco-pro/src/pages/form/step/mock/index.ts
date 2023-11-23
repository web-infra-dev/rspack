import setupMock from "@/utils/setupMock";
import Mock from "mockjs";

setupMock({
	setup: () => {
		// 保存表单数据
		Mock.mock(new RegExp("/api/groupForm"), () => {
			return true;
		});
	}
});
