import { invoke } from "@tauri-apps/api/core";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import { Gvk } from "../../model/k8s";

export type MenuBlueprint = {
    id: string,
    items: MenuSection[]
};

export type MenuSection = {
    title?: string,
    items: MenuItem[]
}

export type ActionButton = { kind: "ActionButton", data: { title: string, dangerous: boolean, actionRef: string } };
export type Separator = { kind: "Separator", data: undefined };
export type MenuItem = ActionButton | Separator;

export default function createResourceMenustack(contextSource: KubeContextSource, gvk: Gvk, namespace: string, name: string) {
    return invoke<MenuBlueprint>("create_resource_menustack", {
        contextSource,
        gvk,
        namespace,
        name
    });
}