import { invoke } from "@tauri-apps/api/core";
import { LogicalPosition } from "@tauri-apps/api/dpi";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import { Gvk } from "../../model/k8s";

export const popupKubernetesResourceMenu = (contextSource: KubeContextSource, tabId: string, namespace: string, name: string, gvk: Gvk, position: LogicalPosition): Promise<null> => {
    return invoke<null>("popup_kubernetes_resource_menu", { contextSource, namespace, name, gvk, position, tabId });
}
