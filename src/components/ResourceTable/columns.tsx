import dayjs from "dayjs";
import { ReactNode } from "react";
import { type ContainerStatus, type NodeAddress } from 'kubernetes-types/core/v1';

export type ColumnDefinition = [string, { description?: string, jsonPath: string, transform?: (value: any) => string | ReactNode }];

export const genericNamespacedResource: ColumnDefinition[] = [
    ['Name', { jsonPath: '$.metadata.name' }],
    ['Namespace', { jsonPath: '$.metadata.namespace' }],
    ['Age', { jsonPath: '$.metadata.creationTimestamp', transform: (timestamp: string) => dayjs().to(dayjs(timestamp), true) }]
];

export const genericNonNamespacedResource: ColumnDefinition[] = [
    ['Name', { jsonPath: '$.metadata.name' }],
    ['Age', { jsonPath: '$.metadata.creationTimestamp', transform: (timestamp: string) => dayjs().to(dayjs(timestamp), true) }]
];

export const coreNodes: ColumnDefinition[] = [
    ['Name', { jsonPath: '$.metadata.name' }],
    ['OS image', { jsonPath: '$.status.nodeInfo.osImage' }],
    ['Internal IP', {
        jsonPath: '$.status.addresses', transform: (addresses: NodeAddress[][]) => {
            return addresses[0].find(address => address.type === 'InternalIP')?.address;
        }
    }],
    ['Architecture', { jsonPath: '$.status.nodeInfo.architecture' }],
    ['Kubelet version', { jsonPath: '$.status.nodeInfo.kubeletVersion' }],
    ['Age', { jsonPath: '$.metadata.creationTimestamp', transform: (timestamp: string) => dayjs().to(dayjs(timestamp), true) }]
];

export const corePods: ColumnDefinition[] = [
    ['Name', { jsonPath: '$.metadata.name' }],
    ['Namespace', { jsonPath: '$.metadata.namespace' }],
    ['Containers', {
        jsonPath: '$.status.containerStatuses[*].', transform: (statuses: ContainerStatus[]) => {
            const boxes = statuses.map((status) => {
                if (status.ready) {
                    return <span style={{ color: status.ready ? 'lightgreen' : 'orange' }}>■</span>
                }

                const state = status.state ? Object.keys(status.state)[0] : '';

                if (state === 'running') return <span style={{ color: 'orange' }}>■</span>;
                return <span style={{ color: 'darkgrey' }}>□</span>
            });

            return <>{boxes.map((box: ReactNode) => <>{box}{"\u00A0"}</>)}</>;
        }
    }],
    ['Restarts', { jsonPath: '$.status.containerStatuses[*].restartCount', transform: (v: number[]) => v.reduce((a, b) => a + b, 0).toString() }],
    ['Node', { jsonPath: '$.spec.nodeName' }],
    ['Status', {
        jsonPath: '$.status.phase', transform: (phase: string) => {
            const colors = {
                Running: "lightgreen",
                Succeeded: "lightgreen",
                Failed: "red",
                Pending: "orange"
            };

            const color = colors[phase[0] as keyof typeof colors];

            return (<span style={{ color: color || 'initial' }}>{phase[0]}</span>);
        }
    }],
    ['Age', { jsonPath: '$.metadata.creationTimestamp', transform: (timestamp: string) => dayjs().to(dayjs(timestamp), true) }]
];
