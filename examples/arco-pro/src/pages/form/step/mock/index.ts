import Mock from 'mockjs';
import setupMock from '@/utils/setupMock';

setupMock({
  setup: () => {
    // 保存表单数据
    Mock.mock(new RegExp('/api/groupForm'), () => {
      return true;
    });
  },
});
