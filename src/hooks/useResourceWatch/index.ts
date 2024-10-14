import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Gvk, KubernetesClient } from "../../model/k8s";

type OkData = { "Ok": string };
type ErrData = { "Err": { "message": string } };
type ColumnData = (OkData | ErrData)[];

type Payload = {
    uid: string,
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

export type ResourceViewData = { [key: string]: ColumnData };

export default function useKubernetesResourceWatch(kubernetesClient: KubernetesClient | undefined, gvk: Gvk | undefined): [string[], ResourceViewData] {
    const [columnTitles, setColumnTitles] = useState<string[]>([]);
    const [resources, setResources] = useState<ResourceViewData>({});

    useEffect(() => {
        if (gvk === undefined) return;
        if (kubernetesClient === undefined) return;

        setResources({});
        setColumnTitles([]);

        const channel = new Channel<WatchEvent>();

        channel.onmessage = (message) => {
            if (message.event === 'created') {
                const { uid, columns } = message.data;
                setResources(datasets => ({
                    ...datasets,
                    [uid]: columns
                }));
            }
            else if (message.event === 'updated') {
                const { uid, columns } = message.data;
                setResources(datasets => ({
                    ...datasets,
                    [uid]: columns
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
        }

        (invoke('watch_gvk_with_view', { clientId: kubernetesClient.id, gvk, channel }) as Promise<string[]>)
            .then(titles => setColumnTitles(titles))
            .catch(e => alert(e));

        return () => {
            invoke('cleanup_channel', { id: channel.id })
        }
    }, [gvk, kubernetesClient]);

    return [columnTitles, resources];
}
