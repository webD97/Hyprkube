import { invoke } from "@tauri-apps/api/core";
import { LogicalPosition } from "@tauri-apps/api/dpi";
import { Gvk } from "../../model/k8s";

export const popupKubernetesResourceMenu = (clientId: string, namespace: string, name: string, gvk: Gvk, position: LogicalPosition): Promise<null> => {
    return invoke<null>("popup_kubernetes_resource_menu", { clientId, namespace, name, gvk, position });
}
