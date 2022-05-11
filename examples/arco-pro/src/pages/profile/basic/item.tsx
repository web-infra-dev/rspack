import React, { CSSProperties } from 'react';
import useLocale from '@/utils/useLocale';
import { Descriptions, Card, Skeleton } from '@arco-design/web-react';
import locale from './locale';

interface ProfileItemProps {
  title: string;
  data: any;
  style?: CSSProperties;
  type: 'origin' | 'current';
  loading?: boolean;
}

function ProfileItem(props: ProfileItemProps) {
  const t = useLocale(locale);
  const { title, data, type, loading } = props;
  const blockDataList: {
    title: string;
    data: {
      label: string;
      value: string;
    }[];
  }[] = [];

  blockDataList.push({
    title: t[`basicProfile.title.${type}Video`],
    data: [
      {
        label: t['basicProfile.label.video.mode'],
        value: data?.video?.mode || '-',
      },
      {
        label: t['basicProfile.label.video.acquisition.resolution'],
        value: data?.video?.acquisition.resolution || '-',
      },
      {
        label: t['basicProfile.label.video.acquisition.frameRate'],
        value: `${data?.video?.acquisition.frameRate || '-'} fps`,
      },
      {
        label: t['basicProfile.label.video.encoding.resolution'],
        value: data?.video?.encoding.resolution || '-',
      },
      {
        label: t['basicProfile.label.video.encoding.rate.min'],
        value: `${data?.video?.encoding.rate.min || '-'} bps`,
      },
      {
        label: t['basicProfile.label.video.encoding.rate.max'],
        value: `${data?.video?.encoding.rate.max || '-'} bps`,
      },
      {
        label: t['basicProfile.label.video.encoding.rate.default'],
        value: `${data?.video?.encoding.rate.default || '-'} bps`,
      },
      {
        label: t['basicProfile.label.video.encoding.frameRate'],
        value: `${data?.video?.encoding.frameRate || '-'} fpx`,
      },
      {
        label: t['basicProfile.label.video.encoding.profile'],
        value: data?.video?.encoding.profile || '-',
      },
    ],
  });

  blockDataList.push({
    title: t[`basicProfile.title.${type}Audio`],
    data: [
      {
        label: t['basicProfile.label.audio.mode'],
        value: data?.audio?.mode || '-',
      },
      {
        label: t['basicProfile.label.audio.acquisition.channels'],
        value: `${data?.audio?.acquisition.channels || '-'} ${
          t['basicProfile.unit.audio.channels']
        }`,
      },
      {
        label: t['basicProfile.label.audio.encoding.channels'],
        value: `${data?.audio?.encoding.channels || '-'} ${
          t['basicProfile.unit.audio.channels']
        }`,
      },
      {
        label: t['basicProfile.label.audio.encoding.rate'],
        value: `${data?.audio?.encoding.rate || '-'} kbps`,
      },
      {
        label: t['basicProfile.label.audio.encoding.profile'],
        value: data?.audio?.encoding.profile || '-',
      },
    ],
  });

  return (
    <Card>
      <div>
        {blockDataList.map(({ title: blockTitle, data: blockData }, index) => (
          <Descriptions
            key={`${index}`}
            colon=":"
            labelStyle={{ textAlign: 'right', width: 200, paddingRight: 10 }}
            valueStyle={{ width: 400 }}
            title={blockTitle}
            data={
              loading
                ? blockData.map((item) => ({
                    ...item,
                    value: (
                      <Skeleton
                        text={{ rows: 1, style: { width: '200px' } }}
                        animation
                      />
                    ),
                  }))
                : blockData
            }
            style={index > 0 ? { marginTop: '20px' } : {}}
          />
        ))}
      </div>
    </Card>
  );
}

export default ProfileItem;
