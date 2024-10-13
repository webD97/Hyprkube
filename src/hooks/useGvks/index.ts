import { useEffect, useState } from "react";
import { discoverGvks } from "../../api/KubernetesClient";
import { KubernetesClient } from "../../model/k8s"

export const useGvks = (kubernetesClient: KubernetesClient | undefined): { [key: string]: [string, string] } => {
    const [gvks, setGvks] = useState<{ [key: string]: [string, string] }>({});

    useEffect(() => {
        if (!kubernetesClient) return;
        discoverGvks(kubernetesClient).then(result => setGvks(result.gvks));
    }, [kubernetesClient]);

    return gvks;
}
