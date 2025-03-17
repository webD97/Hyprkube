import { invoke } from "@tauri-apps/api/core";

export default function restartStatefulSet(clientId: string, namespace: string, name: string) {
    return invoke<void>('restart_statefulset', {
        clientId,
        namespace, name
    });
}