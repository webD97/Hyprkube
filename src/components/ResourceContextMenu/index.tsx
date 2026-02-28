import { ItemType } from "antd/es/menu/interface";
import { PropsWithChildren, useRef } from "react";
import callMenustackAction from "../../api/callMenuStackAction";
import createResourceMenustack from "../../api/createResourceMenustack";
import dropResourceMenustack from "../../api/dropResourceMenustack";
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
    // If a Promise is put in here, it must be awaited before dropping the menu stack
    // to avoid race conditions where dropping might be faster than calling the action.
    const lockRef = useRef<Promise<void>>(null);
    const menuIdRef = useRef<string>(null);

    return (
        <LazyDropdown
            fetchItems={async () => {
                const blueprint = await createResourceMenustack(contextSource, gvk, namespace, name);

                menuIdRef.current = blueprint.id;

                return blueprint.items.flatMap(({ title, items }, idx) => {
                    const children: ItemType[] = items.map(({ kind, data }) => {
                        switch (kind) {
                            case "Separator": {
                                if (idx >= blueprint.items.length - 1) {
                                    return undefined
                                }
                                return ({ type: "divider" }) satisfies ItemType;
                            }
                            case "ActionButton": {
                                return ({
                                    type: "item",
                                    key: data.actionRef,
                                    label: data.title,
                                    danger: data.dangerous,
                                    onClick: () => {
                                        lockRef.current = callMenustackAction(contextSource, blueprint.id, data.actionRef)
                                            .finally(() => lockRef.current = null);
                                    }
                                }) satisfies ItemType;
                            }
                        }
                    }).filter(i => i !== undefined);

                    // Sections without a title should not be wrapped in a group to avoid weird layout
                    if (!title) {
                        return children;
                    }

                    return [{
                        type: "group",
                        label: title,
                        children
                    }];
                });
            }}
            onSubmenuActivated={() => Promise.resolve([])}
            onClose={() => {
                void (async () => {
                    if (lockRef.current) {
                        await lockRef.current;
                    }
                    await dropResourceMenustack(contextSource, menuIdRef.current!);
                })();
            }}
        >
            {children}
        </LazyDropdown>
    );
}
