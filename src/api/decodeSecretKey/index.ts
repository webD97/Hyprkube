import { invoke } from "@tauri-apps/api/core";

export default function decodeSecretKey(
    clientId: string,
    namespace: string,
    name: string,
    key: string
) {
    return invoke<string>('decode_secret_key', {
        clientId, namespace, name, key
    })
}