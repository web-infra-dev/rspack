import Mock from 'mockjs';
import setupMock from '@/utils/setupMock';

setupMock({
  setup: () => {
    // 我的项目
    Mock.mock(new RegExp('/api/user/projectList'), () => {
      const contributors = [
        {
          name: '秦臻宇',
          email: 'qingzhenyu@arco.design',
          avatar:
            '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/a8c8cdb109cb051163646151a4a5083b.png~tplv-uwbnlip3yd-webp.webp',
        },
        {
          name: '于涛',
          email: 'yuebao@arco.design',
          avatar:
            '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/a8c8cdb109cb051163646151a4a5083b.png~tplv-uwbnlip3yd-webp.webp',
        },
        {
          name: '宁波',
          email: 'ningbo@arco.design',
          avatar:
            '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/3ee5f13fb09879ecb5185e440cef6eb9.png~tplv-uwbnlip3yd-webp.webp',
        },
        {
          name: '郑曦月',
          email: 'zhengxiyue@arco.design',
          avatar:
            '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/8361eeb82904210b4f55fab888fe8416.png~tplv-uwbnlip3yd-webp.webp',
        },
        {
          name: '宁波',
          email: 'ningbo@arco.design',
          avatar:
            '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/3ee5f13fb09879ecb5185e440cef6eb9.png~tplv-uwbnlip3yd-webp.webp',
        },
      ];
      return new Array(6).fill(null).map((_item, index) => ({
        id: index,
        enTitle: [
          'Arco Design System',
          'The Volcano Engine',
          'OCR text recognition',
          'Content resource management',
          'Toutiao content management',
          'Intelligent Robot Project',
        ][index],
        title: [
          '企业级产品设计系统',
          '火山引擎智能应用',
          'OCR文本识别',
          '内容资源管理',
          '今日头条内容管理',
          '智能机器人',
        ][index],
        contributors,
        contributorsLength: Mock.Random.natural(5, 100),
      }));
    });

    // 我的团队
    Mock.mock(new RegExp('/api/users/teamList'), () => {
      return new Array(4).fill(null).map((_, index) => ({
        name: [
          '火山引擎智能应用团队',
          '企业级产品设计团队',
          '前端/UE小分队',
          '内容识别插件小分队',
        ][index],
        avatar: [
          '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/a8c8cdb109cb051163646151a4a5083b.png~tplv-uwbnlip3yd-webp.webp',
          '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/3ee5f13fb09879ecb5185e440cef6eb9.png~tplv-uwbnlip3yd-webp.webp',
          '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/3ee5f13fb09879ecb5185e440cef6eb9.png~tplv-uwbnlip3yd-webp.webp',
          '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/8361eeb82904210b4f55fab888fe8416.png~tplv-uwbnlip3yd-webp.webp',
        ][index],
        members: Mock.Random.natural(1, 1000),
      }));
    });

    // 最新动态
    Mock.mock(new RegExp('/api/user/latestNews'), () => {
      return new Array(8).fill(null).map((_item, index) => ({
        id: index,
        title: '王多鱼审核了图文内容： 2021年，你过得怎么样？',
        description:
          '新华网年终特别策划：《这一年，你过得怎么样？》回访那些你最熟悉的“陌生人”带你重温这难忘的2021年回顾我们共同记忆中的生动故事！',
        avatar:
          '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/a8c8cdb109cb051163646151a4a5083b.png~tplv-uwbnlip3yd-webp.webp',
      }));
    });

    // 站内通知
    Mock.mock(new RegExp('/api/user/notice'), () => {
      return [];
    });
  },
});
