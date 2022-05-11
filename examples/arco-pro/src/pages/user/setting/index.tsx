import React, { useState } from 'react';
import { useSelector } from 'react-redux';
import { Card, Tabs } from '@arco-design/web-react';
import useLocale from '@/utils/useLocale';
import locale from './locale';
import InfoHeader from './header';
import InfoForm from './info';
import Security from './security';
import './mock';
import Verified from './verified';

function UserInfo() {
  const t = useLocale(locale);
  const userInfo = useSelector((state: any) => state.userInfo);
  const loading = useSelector((state: any) => state.userLoading);
  const [activeTab, setActiveTab] = useState('basic');
  return (
    <div>
      <Card style={{ padding: '14px 20px' }}>
        <InfoHeader userInfo={userInfo} loading={loading} />
      </Card>
      <Card style={{ marginTop: '16px' }}>
        <Tabs activeTab={activeTab} onChange={setActiveTab} type="rounded">
          <Tabs.TabPane key="basic" title={t['userSetting.title.basicInfo']}>
            <InfoForm loading={loading} />
          </Tabs.TabPane>
          <Tabs.TabPane key="security" title={t['userSetting.title.security']}>
            <Security />
          </Tabs.TabPane>
          <Tabs.TabPane key="verified" title={t['userSetting.label.verified']}>
            <Verified />
          </Tabs.TabPane>
        </Tabs>
      </Card>
    </div>
  );
}

export default UserInfo;
