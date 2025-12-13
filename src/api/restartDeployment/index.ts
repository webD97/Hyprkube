import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";

export default function restartDeployment(contextSource: KubeContextSource, namespace: string, name: string) {
    return invoke<void>('restart_deployment', {
        contextSource,
        namespace, name
    });
}