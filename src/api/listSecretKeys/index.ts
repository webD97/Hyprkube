import { invoke } from "@tauri-apps/api/core";

export default function listSecretKeys(
    clientId: string,
    namespace: string,
    name: string
) {
    return invoke<string[]>('list_secret_keys', {
        clientId, namespace, name
    })
}