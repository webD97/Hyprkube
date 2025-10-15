import { Channel, invoke } from "@tauri-apps/api/core";
import dayjs from "dayjs";
import { useEffect, useState } from "react";
import { Gvk } from "../../model/k8s";

type ResourceFieldComponent =
    {
        PlainString: string
    }
    |
    {
        ColoredString: {
            string: string,
            color: string
        }
    }
    |
    {
        ColoredBox: {
            color: string,
        }
    }
    |
    {
        Hyperlink: {
            url: string,
            display_text: string
        }
    }
    |
    {
        RelativeTime: {
            iso: string,
        }
    };

export type ResourceField = {
    components: ResourceFieldComponent[],
    sortableValue: string
};

export type OkData = { "Ok": ResourceFieldComponent[] };
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
    columns: ResourceField[]
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
                return ({ components: [{ PlainString: column.Err }], sortableValue: column.Err });
            }
            if ("Ok" in column) {
                const components = column.Ok;
                return ({
                    components,
                    sortableValue: components.map(component => {
                        if ("RelativeTime" in component) {
                            return dayjs(component.RelativeTime.iso).unix();
                        }
                        return component[Object.keys(component)[0] as keyof ResourceFieldComponent];
                    }).join()
                });
            }

            // We cannot reach this
            return ({ components: [], sortableValue: "" });
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
