import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Gvk } from "../../model/k8s";

export function useDefaultNamespace(profile: string, gvk: Gvk) {
    const [defaultNamespace, setDefaultNamespace] = useState('default');

    useEffect(() => {
        invoke<string>('get_default_namespace', {
            profile, gvk
        })
            .then(setDefaultNamespace)
            .catch(e => console.log(e));
    }, [gvk, profile]);

    return defaultNamespace;
}