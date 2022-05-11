import Mock from 'mockjs';
import setupMock from '@/utils/setupMock';

setupMock({
  setup: () => {
    Mock.mock(new RegExp('/api/basicProfile'), () => {
      return {
        status: 2,
        video: {
          mode: '自定义',
          acquisition: {
            resolution: '720*1280',
            frameRate: 15,
          },
          encoding: {
            resolution: '720*1280',
            rate: {
              min: 300,
              max: 800,
              default: 1500,
            },
            frameRate: 15,
            profile: 'high',
          },
        },
        audio: {
          mode: '自定义',
          acquisition: {
            channels: 8,
          },
          encoding: {
            channels: 8,
            rate: 128,
            profile: 'ACC-LC',
          },
        },
      };
    });

    Mock.mock(new RegExp('/api/adjustment'), () => {
      return new Array(2).fill('0').map(() => ({
        contentId: `${Mock.Random.pick([
          '视频类',
          '音频类',
        ])}${Mock.Random.natural(1000, 9999)}`,
        content: '视频参数变更，音频参数变更',
        status: Mock.Random.natural(0, 1),
        updatedTime: Mock.Random.datetime('yyyy-MM-dd HH:mm:ss'),
      }));
    });
  },
});
