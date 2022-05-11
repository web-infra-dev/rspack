import React from 'react';
import {
  List,
  Avatar,
  Typography,
  Button,
  Space,
  Result,
  Tag,
} from '@arco-design/web-react';
import useLocale from '../../utils/useLocale';
import styles from './style/index.module.less';

export interface MessageItemData {
  id: string;
  title: string;
  subTitle?: string;
  avatar?: string;
  content: string;
  time?: string;
  status: number;
  tag?: {
    text?: string;
    color?: string;
  };
}

export type MessageListType = MessageItemData[];

interface MessageListProps {
  data: MessageItemData[];
  unReadData: MessageItemData[];
  onItemClick?: (item: MessageItemData, index: number) => void;
  onAllBtnClick?: (
    unReadData: MessageItemData[],
    data: MessageItemData[]
  ) => void;
}

function MessageList(props: MessageListProps) {
  const t = useLocale();
  const { data, unReadData } = props;

  function onItemClick(item: MessageItemData, index: number) {
    if (item.status) return;
    props.onItemClick && props.onItemClick(item, index);
  }

  function onAllBtnClick() {
    props.onAllBtnClick && props.onAllBtnClick(unReadData, data);
  }

  return (
    <List
      noDataElement={<Result status="404" subTitle={t['message.empty.tips']} />}
      footer={
        <div className={styles.footer}>
          <div className={styles['footer-item']}>
            <Button type="text" size="small" onClick={onAllBtnClick}>
              {t['message.allRead']}
            </Button>
          </div>
          <div className={styles['footer-item']}>
            <Button type="text" size="small">
              {t['message.seeMore']}
            </Button>
          </div>
        </div>
      }
    >
      {data.map((item, index) => (
        <List.Item
          key={item.id}
          actionLayout="vertical"
          style={{
            opacity: item.status ? 0.5 : 1,
          }}
        >
          <div
            style={{
              cursor: 'pointer',
            }}
            onClick={() => {
              onItemClick(item, index);
            }}
          >
            <List.Item.Meta
              avatar={
                item.avatar && (
                  <Avatar shape="circle" size={36}>
                    <img src={item.avatar} />
                  </Avatar>
                )
              }
              title={
                <div className={styles['message-title']}>
                  <Space size={4}>
                    <span>{item.title}</span>
                    <Typography.Text type="secondary">
                      {item.subTitle}
                    </Typography.Text>
                  </Space>
                  {item.tag && item.tag.text ? (
                    <Tag color={item.tag.color}>{item.tag.text}</Tag>
                  ) : null}
                </div>
              }
              description={
                <div>
                  <Typography.Paragraph style={{ marginBottom: 0 }} ellipsis>
                    {item.content}
                  </Typography.Paragraph>
                  <Typography.Text type="secondary" style={{ fontSize: 12 }}>
                    {item.time}
                  </Typography.Text>
                </div>
              }
            />
          </div>
        </List.Item>
      ))}
    </List>
  );
}

export default MessageList;
