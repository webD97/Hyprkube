import React from 'react';

import classes from './component.module.css';
import { LazyLog, ScrollFollow } from '@melloware/react-logviewer';
import { usePodLogs } from '../../hooks/usePodLogs';
import { KubernetesClient } from '../../model/k8s';

export interface LogPanelProps {
    kubernetesClient: KubernetesClient|undefined,
    namespace: string,
    name: string
}

const LogPanel: React.FC<LogPanelProps> = (props) => {
    const {
        kubernetesClient, namespace, name
    } = props;

    const text = usePodLogs(kubernetesClient, namespace, name);

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
