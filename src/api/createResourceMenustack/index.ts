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

export type ResourceRef = {
    apiVersion: string,
    kind: string,
    namespace?: string,
    name: string
}

export type SubMenu = { kind: "SubMenu", data: { title: string, items: MenuItem[] } };
export type ResourceSubMenu = { kind: "ResourceSubMenu", data: { title: string, resourceRef: ResourceRef } };
export type ActionButton = { kind: "ActionButton", data: { title: string, dangerous: boolean, confirm?: string, actionRef: string } };
export type Separator = { kind: "Separator", data: undefined };
export type MenuItem = ActionButton | SubMenu | ResourceSubMenu | Separator;

export default function createResourceMenustack(parentMenu: string | undefined, contextSource: KubeContextSource, tabId: string, gvk: Gvk, namespace: string, name: string) {
    return invoke<MenuBlueprint>("create_resource_menustack", {
        parentMenu,
        contextSource,
        tabId,
        gvk,
        namespace,
        name
    });
}