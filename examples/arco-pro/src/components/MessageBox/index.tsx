import React, { useEffect, useState } from 'react';
import axios from 'axios';
import groupBy from 'lodash/groupBy';
import {
  Trigger,
  Badge,
  Tabs,
  Avatar,
  Spin,
  Button,
} from '@arco-design/web-react';
import {
  IconMessage,
  IconCustomerService,
  IconFile,
  IconDesktop,
} from '@arco-design/web-react/icon';
import useLocale from '../../utils/useLocale';
import MessageList, { MessageListType } from './list';
import styles from './style/index.module.less';

function DropContent() {
  const t = useLocale();
  const [loading, setLoading] = useState(false);
  const [groupData, setGroupData] = useState<{
    [key: string]: MessageListType;
  }>({});
  const [sourceData, setSourceData] = useState<MessageListType>([]);

  function fetchSourceData(showLoading = true) {
    showLoading && setLoading(true);
    axios
      .get('/api/message/list')
      .then((res) => {
        setSourceData(res.data);
      })
      .finally(() => {
        showLoading && setLoading(false);
      });
  }

  function readMessage(data: MessageListType) {
    const ids = data.map((item) => item.id);
    axios
      .post('/api/message/read', {
        ids,
      })
      .then(() => {
        fetchSourceData();
      });
  }

  useEffect(() => {
    fetchSourceData();
  }, []);

  useEffect(() => {
    const groupData: { [key: string]: MessageListType } = groupBy(
      sourceData,
      'type'
    );
    setGroupData(groupData);
  }, [sourceData]);

  const tabList = [
    {
      key: 'message',
      title: t['message.tab.title.message'],
      titleIcon: <IconMessage />,
    },
    {
      key: 'notice',
      title: t['message.tab.title.notice'],
      titleIcon: <IconCustomerService />,
    },
    {
      key: 'todo',
      title: t['message.tab.title.todo'],
      titleIcon: <IconFile />,
      avatar: (
        <Avatar style={{ backgroundColor: '#0FC6C2' }}>
          <IconDesktop />
        </Avatar>
      ),
    },
  ];

  return (
    <div className={styles['message-box']}>
      <Spin loading={loading} style={{ display: 'block' }}>
        <Tabs
          overflow="dropdown"
          type="rounded"
          defaultActiveTab="message"
          destroyOnHide
          extra={
            <Button type="text" onClick={() => setSourceData([])}>
              {t['message.empty']}
            </Button>
          }
        >
          {tabList.map((item) => {
            const { key, title, avatar } = item;
            const data = groupData[key] || [];
            const unReadData = data.filter((item) => !item.status);
            return (
              <Tabs.TabPane
                key={key}
                title={
                  <span>
                    {title}
                    {unReadData.length ? `(${unReadData.length})` : ''}
                  </span>
                }
              >
                <MessageList
                  data={data}
                  unReadData={unReadData}
                  onItemClick={(item) => {
                    readMessage([item]);
                  }}
                  onAllBtnClick={(unReadData) => {
                    readMessage(unReadData);
                  }}
                />
              </Tabs.TabPane>
            );
          })}
        </Tabs>
      </Spin>
    </div>
  );
}

function MessageBox({ children }) {
  return (
    <Trigger
      trigger="hover"
      popup={() => <DropContent />}
      position="br"
      unmountOnExit={false}
      popupAlign={{ bottom: 4 }}
    >
      <Badge count={9} dot>
        {children}
      </Badge>
    </Trigger>
  );
}

export default MessageBox;
