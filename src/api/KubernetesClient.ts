import { invoke } from "@tauri-apps/api/core";
import { KubernetesClient } from "../model/k8s";

export function getDefaultKubernetesClient() {
    return invoke('initialize_kube_client') as Promise<KubernetesClient>;
}
