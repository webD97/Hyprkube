import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { WatchEvent } from "../../api/WatchEvent";
import { KubernetesApiObject } from "../../model/k8s";

export default function useKubernetesResourceWatch<K extends KubernetesApiObject>(tauriCommandName: string) {
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

        invoke(tauriCommandName, { channel });
    }, []);

    return pods;
}
