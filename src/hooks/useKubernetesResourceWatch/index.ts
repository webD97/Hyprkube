import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { WatchEvent } from "../../api/WatchEvent";
import { ObjectMeta } from "kubernetes-types/meta/v1";

interface GenericResource {
    apiVersion?: string,
    kind?: string,
    metadata?: ObjectMeta,
}

export default function useKubernetesResourceWatch<K extends GenericResource>(group: string, version: string, kind: string) {
    const [pods, setPods] = useState<Array<K>>([]);

    useEffect(() => {
        const channel = new Channel<WatchEvent<K>>();

        channel.onmessage = (message) => {
            const newPod = message.data.repr as K;

            if (message.event === 'created') {
                setPods(pods => [
                    ...pods.filter(pod => pod.metadata?.uid !== newPod.metadata?.uid),
                    message.data.repr as K
                ]);
            }
            else if (message.event === 'updated') {
                setPods(pods => [
                    ...pods.filter(pod => pod.metadata?.uid !== newPod.metadata?.uid),
                    message.data.repr as K
                ]);
            }
            else if (message.event === 'deleted') {
                setPods(pods => [
                    ...pods.filter(pod => pod.metadata?.uid !== newPod.metadata?.uid)
                ]);
            }
        }

        invoke('kube_watch_gvk', { group, version, kind, channel});
    }, []);

    return pods;
}
