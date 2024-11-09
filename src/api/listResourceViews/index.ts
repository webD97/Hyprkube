import { invoke } from "@tauri-apps/api/core";
import { Gvk } from "../../model/k8s";

export type ResourceViewDef = string;

export default async function listResourceViews(
    clientId: string,
    gvk: Gvk,
): Promise<ResourceViewDef[]> {
    return invoke('list_resource_views', {
        ...gvk,
        clientId,
    });
}
