import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";

export default function restartStatefulSet(contextSource: KubeContextSource, namespace: string, name: string) {
    return invoke<void>('restart_statefulset', {
        contextSource,
        namespace, name
    });
}