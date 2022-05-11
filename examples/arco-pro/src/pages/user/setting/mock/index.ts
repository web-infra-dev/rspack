import Mock from 'mockjs';
import setupMock from '@/utils/setupMock';

setupMock({
  setup: () => {
    // 保存个人信息
    Mock.mock(new RegExp('/api/user/saveInfo'), () => {
      return 'ok';
    });

    // 实名认证信息
    Mock.mock(new RegExp('/api/user/verified/enterprise'), () => {
      return Mock.mock({
        accountType: '企业账号',
        isVerified: true,
        verifiedTime: Mock.Random.datetime('yyyy-MM-dd HH:mm:ss'),
        legalPersonName: Mock.Random.cfirst() + '**',
        certificateType: '中国身份证',
        certificationNumber: /[1-9]{3}[*]{12}[0-9]{3}/,
        enterpriseName: Mock.Random.ctitle(),
        enterpriseCertificateType: '企业营业执照',
        organizationCode: /[1-9]{1}[*]{7}[0-9]{1}/,
      });
    });

    Mock.mock(new RegExp('/api/user/verified/authList'), () => {
      return new Array(3).fill('0').map(() => ({
        authType: '企业证件认证',
        authContent: `企业证件认证，法人姓名${Mock.Random.cfirst()}**`,
        authStatus: Mock.Random.natural(0, 1),
        createdTime: Mock.Random.datetime('yyyy-MM-dd HH:mm:ss'),
      }));
    });
  },
});
