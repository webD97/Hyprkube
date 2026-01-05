import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Gvk } from "../../model/k8s";
import { KubeContextSource } from "../useContextDiscovery";

export type ResourceViewDef = string;

export default function useResourceViews(
    contextSource: KubeContextSource,
    gvk: Gvk,
): ResourceViewDef[] {
    const [views, setViews] = useState<ResourceViewDef[]>([]);

    useEffect(() => {
        invoke<ResourceViewDef[]>('list_resource_views', {
            ...gvk,
            contextSource,
        })
            .then(views => setViews(views))
            .catch(e => console.log(e));
    }, [contextSource, gvk]);

    return views;
}
