import { invoke } from "@tauri-apps/api";
import { Namespace as CoreV1Namespace } from 'kubernetes-types/core/v1';
import { useState, useEffect } from "react";

export default function useKubernetesNamespaceList() {
    const [namespaces, setNamespaces] = useState<Array<CoreV1Namespace>>([]);

    useEffect(() => {
        invoke('kube_get_namespaces')
            .then((namespaces) => {
                setNamespaces(namespaces as Array<CoreV1Namespace>);
            })
            .catch((e) => {
                console.log({ e });
            });
    }, []);

    return namespaces;
}
