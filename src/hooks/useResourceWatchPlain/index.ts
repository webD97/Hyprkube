import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Gvk } from "../../model/k8s";

type Payload = {
    uid: string,
    namespace: string,
    name: string,
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
        data: Payload
    }

export default function useKubernetesResourceWatchPlain(kubernetesClientId: string | undefined, gvk: Gvk | undefined, namespace?: string): { [key: string]: Payload } {
    const [resources, setResources] = useState<{ [key: string]: Payload }>({});

    useEffect(() => {
        if (gvk === undefined) return;
        if (kubernetesClientId === undefined) return;

        const channel = new Channel<WatchEvent>();

        channel.onmessage = (message) => {
            if (message.event === 'created') {
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

        invoke('watch_gvk_plain', { clientId: kubernetesClientId, gvk, channel, namespace })
            .catch(e => alert(e));

        return () => {
            invoke('cleanup_channel', { channel });
        };
    }, [gvk, kubernetesClientId]);

    return resources;
}
