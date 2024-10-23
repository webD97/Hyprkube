import React from 'react';

import classes from './component.module.css';
import { LazyLog, ScrollFollow } from '@melloware/react-logviewer';
import { usePodLogs } from '../../hooks/usePodLogs';

export interface LogPanelProps {
    kubernetesClientId: string | undefined,
    namespace: string,
    name: string
}

const LogPanel: React.FC<LogPanelProps> = (props) => {
    const {
        kubernetesClientId, namespace, name
    } = props;

    const text = usePodLogs(kubernetesClientId, namespace, name);

    return (
        <div className={classes.container}>
            <ScrollFollow
                startFollowing={true}
                render={({ follow, onScroll }) => (
                    <LazyLog
                        text={text}
                        enableLineNumbers={false}
                        enableSearch
                        caseInsensitive
                        follow={follow}
                        onScroll={onScroll}
                    />
                )}
            />
        </div>
    );
}

export default LogPanel;
