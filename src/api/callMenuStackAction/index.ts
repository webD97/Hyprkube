import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";

export default function callMenustackAction(contextSource: KubeContextSource, menustackId: string, actionRef: string) {
    return invoke<void>("call_menustack_action", { contextSource, menustackId, actionRef });
}