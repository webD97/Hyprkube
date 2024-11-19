import { invoke } from "@tauri-apps/api/core";

export default async function listPodContainerNames(
    clientId: string,
    namespace: string,
    name: string
): Promise<string[]> {
    return invoke('list_pod_container_names', { clientId, namespace, name });
}