import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Gvk, KubernetesClient } from "../../model/k8s";

type ResourceField =
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

type OkData = { "Ok": ResourceField[] };
type ErrData = { "Err": string };
type ColumnData = (OkData | ErrData)[];

type Payload = {
    uid: string,
    namespace: string,
    name: string,
    columns: ColumnData
}

export type WatchEvent =
    | {
        event: 'created';
        data: Payload
    }
    | {
        event: 'updated';
        data: Payload
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
            titles: string[]
        }
    }

export type ResourceViewData = {
    [key: string]: Payload
};

export default function useKubernetesResourceWatch(kubernetesClient: KubernetesClient | undefined, gvk: Gvk | undefined, viewName: string): [string[], ResourceViewData] {
    const [columnTitles, setColumnTitles] = useState<string[]>([]);
    const [resources, setResources] = useState<ResourceViewData>({});

    useEffect(() => {
        if (gvk === undefined) return;
        if (kubernetesClient === undefined) return;
        if (viewName === '') return;

        const channel = new Channel<WatchEvent>();

        channel.onmessage = (message) => {
            if (message.event === 'announceColumns') {
                setColumnTitles(message.data.titles);
            }
            else if (message.event === 'created') {
                const { uid } = message.data;
                setResources(datasets => ({
                    ...datasets,
                    [uid]: message.data
                }));
            }
            else if (message.event === 'updated') {
                const { uid } = message.data;
                setResources(datasets => ({
                    ...datasets,
                    [uid]: message.data
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
        setColumnTitles([]);

        invoke('watch_gvk_with_view', { clientId: kubernetesClient.id, gvk, channel, viewName })
            .catch(e => alert(e));

        return () => {
            invoke('cleanup_channel', { channel });
        };
    }, [gvk, kubernetesClient, viewName]);

    return [columnTitles, resources];
}
