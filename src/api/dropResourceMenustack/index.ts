import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";

export default function dropResourceMenustack(contextSource: KubeContextSource, menuId: string) {
    return invoke<void>("drop_resource_menustack", { contextSource, menuId });
}
