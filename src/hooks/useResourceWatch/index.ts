import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Gvk } from "../../model/k8s";

type Properties = {
    color?: string,
    title?: string,
}

type CommonFields = {
    properties: Properties | null,
    sortableValue: string
}

export type ViewComponent =
    {
        kind: "Text",
        args: {
            content: string
        }
    } & CommonFields
    |
    {
        kind: "Hyperlink",
        args: {
            url: string,
            display: string
        }
    } & CommonFields
    |
    {
        kind: "RelativeTime",
        args: {
            timestamp: string
        }
    } & CommonFields
    |
    {
        kind: "ColoredBoxes",
        args: {
            boxes: { color: string, properties: Properties | null }[][]
        }
    } & CommonFields
    |
    {
        kind: "ColoredBox",
        args: {
            color: string
        }
    } & CommonFields;

export type OkData = { "Ok": ViewComponent };
export type ErrData = { "Err": string };
export type ColumnData = (OkData | ErrData)[];

type Resource = {
    uid: string,
    namespace: string,
    name: string,
    columns: ColumnData
}

export type DisplayableResource = {
    uid: string,
    namespace: string,
    name: string,
    columns: ViewComponent[]
}

export type ColumnDefinition = {
    title: string,
    filterable: boolean
}

export type WatchEvent =
    | {
        event: 'applied';
        data: Resource
    }
    | {
        event: 'deleted';
        data: {
            uid: string
        }
    }
    | {
        event: 'announceColumns';
        data: {
            columns: ColumnDefinition[]
        }
    }

export type ResourceViewData = {
    [key: string]: DisplayableResource
};

function resourceToDisplayableResource(resource: Resource): DisplayableResource {
    return ({
        namespace: resource.namespace,
        name: resource.name,
        uid: resource.uid,
        columns: resource.columns.map((column) => {
            if ("Err" in column) {
                return ({ kind: "Text", args: { content: column.Err }, properties: null, sortableValue: column.Err });
            }
            if ("Ok" in column) {
                return column.Ok;
            }

            // This should be unreachable
            return ({ kind: "Text", args: { content: "(Unreachable)" }, properties: null, sortableValue: "(Unreachable)" });
        })
    });
}

export default function useKubernetesResourceWatch(kubernetesClientId: string | undefined, gvk: Gvk | undefined, viewName: string, namespace: string): [ColumnDefinition[], ResourceViewData] {
    const [columnDefinitions, setColumnDefinitions] = useState<ColumnDefinition[]>([]);
    const [resources, setResources] = useState<ResourceViewData>({});

    useEffect(() => {
        if (gvk === undefined) return;
        if (kubernetesClientId === undefined) return;
        if (viewName === '') return;

        const channel = new Channel<WatchEvent>();

        channel.onmessage = (message) => {
            if (message.event === 'announceColumns') {
                setColumnDefinitions(message.data.columns);
            }
            else if (message.event === 'applied') {
                const { uid } = message.data;
                setResources(datasets => ({
                    ...datasets,
                    [uid]: resourceToDisplayableResource(message.data)
                }));
            }
            else if (message.event === 'deleted') {
                const { uid } = message.data;
                setResources(datasets => {
                    const newData = ({
                        ...datasets,
                    });

                    delete newData[uid];

                    return newData;
                });
            }
        };

        // We really want to reset the state at this point:
        // eslint-disable-next-line react-hooks/set-state-in-effect
        setResources({});
        setColumnDefinitions([]);

        invoke('watch_gvk_with_view', { clientId: kubernetesClientId, gvk, channel, viewName, namespace })
            .catch(e => {
                if (e === 'BackgroundTaskRejected') return;
                alert("blubb" + e);
            });

        return () => {
            void invoke('cleanup_channel', { channel });
        };
    }, [gvk, kubernetesClientId, viewName, namespace]);

    return [columnDefinitions, resources];
}
