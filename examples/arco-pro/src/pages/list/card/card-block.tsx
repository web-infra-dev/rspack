import React, { useEffect, useState } from 'react';
import cs from 'classnames';
import {
  Button,
  Switch,
  Tag,
  Card,
  Descriptions,
  Typography,
  Dropdown,
  Menu,
  Skeleton,
} from '@arco-design/web-react';
import {
  IconStarFill,
  IconThumbUpFill,
  IconSunFill,
  IconFaceSmileFill,
  IconPenFill,
  IconCheckCircleFill,
  IconCloseCircleFill,
  IconMore,
} from '@arco-design/web-react/icon';
import PermissionWrapper from '@/components/PermissionWrapper';
import useLocale from '@/utils/useLocale';
import locale from './locale';
import { QualityInspection, BasicCard } from './interface';
import styles from './style/index.module.less';

interface CardBlockType {
  type: 'quality' | 'service' | 'rules';
  card: QualityInspection & BasicCard;
  loading?: boolean;
}

const IconList = [
  IconStarFill,
  IconThumbUpFill,
  IconSunFill,
  IconFaceSmileFill,
  IconPenFill,
].map((Tag, index) => <Tag key={index} />);

const { Paragraph } = Typography;

function CardBlock(props: CardBlockType) {
  const { type, card = {} } = props;
  const [visible, setVisible] = useState(false);
  const [status, setStatus] = useState(card.status);
  const [loading, setLoading] = useState(props.loading);

  const t = useLocale(locale);
  const changeStatus = async () => {
    setLoading(true);
    await new Promise((resolve) =>
      setTimeout(() => {
        setStatus(status !== 1 ? 1 : 0);
        resolve(null);
      }, 1000)
    ).finally(() => setLoading(false));
  };

  useEffect(() => {
    setLoading(props.loading);
  }, [props.loading]);

  useEffect(() => {
    if (card.status !== status) {
      setStatus(card.status);
    }
  }, [card.status]);

  const getTitleIcon = () => {
    if (type === 'service' && typeof card.icon === 'number') {
      return (
        <div className={styles.icon}>
          {IconList[card.icon % IconList.length]}
        </div>
      );
    }
    return null;
  };

  const getButtonGroup = () => {
    if (type === 'quality') {
      return (
        <>
          <PermissionWrapper
            requiredPermissions={[
              { resource: /^menu.list.*/, actions: ['read'] },
            ]}
          >
            <Button
              type="primary"
              style={{ marginLeft: '12px' }}
              loading={loading}
            >
              {t['cardList.options.qualityInspection']}
            </Button>
          </PermissionWrapper>

          <PermissionWrapper
            requiredPermissions={[
              { resource: /^menu.list.*/, actions: ['write'] },
            ]}
          >
            <Button loading={loading}>{t['cardList.options.remove']}</Button>
          </PermissionWrapper>
        </>
      );
    }

    if (type === 'service') {
      return (
        <>
          {status === 1 ? (
            <Button loading={loading} onClick={changeStatus}>
              {t['cardList.options.cancel']}
            </Button>
          ) : (
            <Button type="outline" loading={loading} onClick={changeStatus}>
              {status === 0
                ? t['cardList.options.subscribe']
                : t['cardList.options.renewal']}
            </Button>
          )}
        </>
      );
    }

    return (
      <Switch checked={!!status} loading={loading} onChange={changeStatus} />
    );
  };

  const getStatus = () => {
    if (type === 'rules' && status) {
      return (
        <Tag
          color="green"
          icon={<IconCheckCircleFill />}
          className={styles.status}
          size="small"
        >
          {t['cardList.tag.activated']}
        </Tag>
      );
    }
    switch (status) {
      case 1:
        return (
          <Tag
            color="green"
            icon={<IconCheckCircleFill />}
            className={styles.status}
            size="small"
          >
            {t['cardList.tag.opened']}
          </Tag>
        );
      case 2:
        return (
          <Tag
            color="red"
            icon={<IconCloseCircleFill />}
            className={styles.status}
            size="small"
          >
            {t['cardList.tag.expired']}
          </Tag>
        );
      default:
        return null;
    }
  };

  const getContent = () => {
    if (loading) {
      return (
        <Skeleton
          text={{ rows: type !== 'quality' ? 3 : 2 }}
          animation
          className={styles['card-block-skeleton']}
        />
      );
    }
    if (type !== 'quality') {
      return <Paragraph>{card.description}</Paragraph>;
    }
    return (
      <Descriptions
        column={2}
        data={[
          { label: '待质检数', value: card.qualityCount },
          { label: '积压时长', value: `${card.duration}s` },
          { label: '待抽检数', value: card.randomCount },
        ]}
      />
    );
  };

  const className = cs(styles['card-block'], styles[`${type}-card`]);

  return (
    <Card
      bordered={true}
      className={className}
      size="small"
      title={
        loading ? (
          <Skeleton
            animation
            text={{ rows: 1, width: ['100%'] }}
            style={{ width: '120px', height: '24px' }}
            className={styles['card-block-skeleton']}
          />
        ) : (
          <>
            <div
              className={cs(styles.title, {
                [styles['title-more']]: visible,
              })}
            >
              {getTitleIcon()}
              {card.title}
              {getStatus()}
              <Dropdown
                droplist={
                  <Menu>
                    {['操作1', '操作2'].map((item, key) => (
                      <Menu.Item key={key.toString()}>{item}</Menu.Item>
                    ))}
                  </Menu>
                }
                trigger="click"
                onVisibleChange={setVisible}
                popupVisible={visible}
              >
                <div className={styles.more}>
                  <IconMore />
                </div>
              </Dropdown>
            </div>
            <div className={styles.time}>{card.time}</div>
          </>
        )
      }
    >
      <div className={styles.content}>{getContent()}</div>
      <div className={styles.extra}>{getButtonGroup()}</div>
    </Card>
  );
}

export default CardBlock;
