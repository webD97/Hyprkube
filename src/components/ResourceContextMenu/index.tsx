import { ItemType } from "antd/es/menu/interface";
import useModal from "antd/es/modal/useModal";
import { PropsWithChildren, use, useRef } from "react";
import callMenustackAction from "../../api/callMenuStackAction";
import createResourceMenustack, { ActionButton, MenuItem, ResourceRef } from "../../api/createResourceMenustack";
import dropResourceMenustack from "../../api/dropResourceMenustack";
import { MegaTabContext } from "../../contexts/MegaTab";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import { Gvk } from "../../model/k8s";
import LazyDropdown from "../LazyDropdown";

export interface ResourceContextMenuProps {
    contextSource: KubeContextSource,
    gvk: Gvk,
    namespace: string,
    name: string
}

export default function ResourceContextMenu({
    contextSource, gvk, namespace, name, children
}: PropsWithChildren<ResourceContextMenuProps>) {
    const lockRef = useRef<Promise<void>>(null);
    const menuIdRef = useRef<string>(null);

    const { tabIdentifier } = use(MegaTabContext)!;
    const [modal, contextHolder] = useModal();

    function runAction(blueprintId: string, data: ActionButton["data"]) {
        const action = async () => {
            if (data.confirm) {
                const confirmed = await modal.confirm({ content: data.confirm });
                if (!confirmed) return;
            }

            await callMenustackAction(contextSource, blueprintId, data.actionRef);
        };

        lockRef.current = action().finally(() => {
            lockRef.current = null;
        });
    };

    function blueprintToMenuItems(blueprint: Awaited<ReturnType<typeof createResourceMenustack>>): ItemType[] {
        return blueprint.items.flatMap(({ title, items }) => {
            const children = make_menu_items({
                items,
                onClick: (data) => runAction(blueprint.id, data)
            });

            if (!title) return children;

            return [{
                type: "group",
                label: title,
                children
            } satisfies ItemType];
        });
    }

    async function loadTopLevelBlueprint(gvk: Gvk, ns: string, n: string) {
        const blueprint = await createResourceMenustack(
            undefined,
            contextSource,
            tabIdentifier.toString(),
            gvk,
            ns,
            n
        );

        menuIdRef.current = blueprint.id;
        return blueprint;
    };

    async function loadSubMenuBlueprint(gvk: Gvk, ns: string, n: string) {
        const blueprint = await createResourceMenustack(
            menuIdRef.current!,
            contextSource,
            tabIdentifier.toString(),
            gvk,
            ns,
            n
        );

        return blueprint;
    };

    return (
        <>
            <div onClick={(e) => e.stopPropagation()}>
                {contextHolder}
            </div>

            <LazyDropdown
                fetchItems={async () => {
                    const blueprint = await loadTopLevelBlueprint(gvk, namespace, name);
                    return blueprintToMenuItems(blueprint);
                }}

                onSubmenuActivated={async (key) => {
                    const resourceRef = JSON.parse(key.toString()) as ResourceRef;

                    const blueprint = await loadSubMenuBlueprint(
                        {
                            ...parseApiVersion(resourceRef.apiVersion),
                            kind: resourceRef.kind
                        },
                        resourceRef.namespace!,
                        resourceRef.name
                    );

                    return blueprintToMenuItems(blueprint);
                }}

                onClose={() => {
                    void (async () => {
                        if (lockRef.current) await lockRef.current;
                        await dropResourceMenustack(contextSource, menuIdRef.current!);
                    })();
                }}
            >
                {children}
            </LazyDropdown>
        </>
    );
}

function parseApiVersion(apiVersion: string): { group: string, version: string } {
    const parts = apiVersion.split("/");

    if (parts.length === 1) {
        return {
            group: "",
            version: parts[0],
        };
    }

    if (parts.length === 2) {
        return {
            group: parts[0],
            version: parts[1],
        };
    }

    throw new Error(`Invalid Kubernetes apiVersion: ${apiVersion}`);
}

type Args = {
    items: MenuItem[],
    onClick: (data: ActionButton["data"]) => void,
}
function make_menu_items({
    items, onClick
}: Args): ItemType[] {
    return items.map(({ kind, data }) => {
        switch (kind) {
            case "Separator": {
                return ({ type: "divider" }) satisfies ItemType;
            }
            case "ActionButton": {
                return ({
                    type: "item",
                    key: data.actionRef,
                    label: data.confirm ? `${data.title} …` : data.title,
                    danger: data.dangerous,
                    onClick: () => {
                        onClick(data);
                    }
                }) satisfies ItemType;
            }
            case "SubMenu": {
                return ({
                    type: "submenu",
                    key: data.title,
                    label: data.title,
                    children: make_menu_items({ items: data.items, onClick })
                }) satisfies ItemType
            }
            case "ResourceSubMenu": {
                return ({
                    type: "submenu",
                    key: JSON.stringify(data.resourceRef),
                    label: data.title,
                    children: []
                }) satisfies ItemType
            }
        }
    }).filter(i => i !== undefined);
}
