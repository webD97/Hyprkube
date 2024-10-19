import { useEffect, useState } from "react";
import { discoverGvks, DiscoveryResult } from "../../api/KubernetesClient";
import { KubernetesClient } from "../../model/k8s"

export const useGvks = (kubernetesClient: KubernetesClient | undefined): DiscoveryResult | undefined => {
    const [gvks, setGvks] = useState<DiscoveryResult | undefined>();

    useEffect(() => {
        if (!kubernetesClient) return;
        discoverGvks(kubernetesClient).then(result => setGvks(result));
    }, [kubernetesClient]);

    return gvks;
}
