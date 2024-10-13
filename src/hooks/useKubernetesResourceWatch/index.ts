import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { WatchEvent } from "../../api/WatchEvent";
import { GenericResource, Gvk } from "../../model/k8s";

export default function useKubernetesResourceWatch<K extends GenericResource>(kubernetesClient: any|undefined, gvk: Gvk | undefined) {
    const [resources, setResources] = useState<Array<K>>([]);

    useEffect(() => {
        if (gvk === undefined) return;
        if (kubernetesClient === undefined) return;

        setResources([]);

        const channel = new Channel<WatchEvent<K>>();

        channel.onmessage = (message) => {
            const newPod = message.data.repr as K;

            if (message.event === 'created') {
                setResources(pods => [
                    ...pods.filter(pod => pod.metadata?.uid !== newPod.metadata?.uid),
                    message.data.repr as K
                ]);
            }
            else if (message.event === 'updated') {
                setResources(pods => [
                    ...pods.filter(pod => pod.metadata?.uid !== newPod.metadata?.uid),
                    message.data.repr as K
                ]);
            }
            else if (message.event === 'deleted') {
                setResources(pods => [
                    ...pods.filter(pod => pod.metadata?.uid !== newPod.metadata?.uid)
                ]);
            }
        }

        invoke('kube_watch_gvk', { clientId: kubernetesClient.id, group: gvk.group, version: gvk.version, kind: gvk.kind, channel })
            .catch(e => alert(e));

        return () => {
            invoke('cleanup_channel', { id: channel.id })
        }
    }, [gvk, kubernetesClient]);

    return resources;
}
