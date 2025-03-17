import { invoke } from "@tauri-apps/api/core";

export default function restartDeployment(clientId: string, namespace: string, name: string) {
    return invoke<void>('restart_deployment', {
        clientId,
        namespace, name
    });
}