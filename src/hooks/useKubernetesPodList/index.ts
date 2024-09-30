import { invoke } from "@tauri-apps/api";
import { Pod as CoreV1Pod } from 'kubernetes-types/core/v1';
import { useEffect, useState } from "react";

export default function useKubernetesNodeList() {
    const [pods, setPods] = useState<Array<CoreV1Pod>>([]);

    useEffect(() => {
        invoke('kube_get_pods')
            .then((result) => {
                setPods(result as Array<CoreV1Pod>);
            })
            .catch((e) => {
                console.log({ e });
            });
    }, []);

    return pods;
}
