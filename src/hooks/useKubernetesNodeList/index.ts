import { invoke } from "@tauri-apps/api";
import { Node as CoreV1Node } from 'kubernetes-types/core/v1';
import { useEffect, useState } from "react";

export default function useKubernetesNodeList() {
    const [nodes, setNodes] = useState<Array<CoreV1Node>>([]);

    useEffect(() => {
        invoke('kube_get_nodes')
            .then((result) => {
                setNodes(result as Array<CoreV1Node>);
            })
            .catch((e) => {
                console.log({ e });
            });
    }, []);

    return nodes;
}
