import { invoke } from "@tauri-apps/api/core";
import { Gvk } from "../../model/k8s";

export async function deleteResource(
    clientId: string,
    gvk: Gvk,
    namespace: string,
    name: string
) {
    return invoke('delete_resource', { clientId, gvk, namespace, name, dryRun: false });
}