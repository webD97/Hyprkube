import dayjs from "dayjs";
import { ReactNode } from "react";

export type ColumnDefinition = [string, { jsonPath: string, transform?: (value: any) => string | ReactNode }];

export const genericNamespacedResource: ColumnDefinition[] = [
    ['Name', { jsonPath: '$.metadata.name' }],
    ['Namespace', { jsonPath: '$.metadata.namespace' }],
    ['Age', { jsonPath: '$.metadata.creationTimestamp', transform: (timestamp: string) => dayjs().to(dayjs(timestamp), true) }]
];

export const genericNonNamespacedResource: ColumnDefinition[] = [
    ['Name', { jsonPath: '$.metadata.name' }],
    ['Age', { jsonPath: '$.metadata.creationTimestamp', transform: (timestamp: string) => dayjs().to(dayjs(timestamp), true) }]
];

export const corePods: ColumnDefinition[] = [
    ['Name', { jsonPath: '$.metadata.name' }],
    ['Namespace', { jsonPath: '$.metadata.namespace' }],
    // ['Containers', { jsonPath: '$.spec.containers.length' }],
    ['Containers', {
        jsonPath: '$.status.containerStatuses[*].state', transform: (state) => {
            const boxes = state.map((status: any) => Object.keys(status)[0]).map((state: string) => {
                if (state === 'terminated') return <span style={{color: 'darkgrey'}}>□</span>
                if (state === 'running') return <span style={{color: 'lightgreen'}}>■</span>
            });

            return <>{boxes.map((box: ReactNode) => <>{box}{"\u00A0"}</>)}</>;
        }
    }],
    ['Restarts', { jsonPath: '$.status.containerStatuses[*].restartCount', transform: (v: number[]) => v.reduce((a, b) => a + b, 0).toString() }],
    ['Node', { jsonPath: '$.spec.nodeName' }],
    ['Status', {
        jsonPath: '$.status.phase', transform: (phase) => {
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
