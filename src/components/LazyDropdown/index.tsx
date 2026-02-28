import { Dropdown } from "antd";
import { ItemType, SubMenuType } from "antd/es/menu/interface";
import { ReactNode, useState } from "react";

export interface LazyDropdownProps {
    fetchItems: () => Promise<ItemType[]>,
    onSubmenuActivated: (key: React.Key) => Promise<ItemType[]>,
    children: ReactNode,
    onClose?: () => void,
}

export default function LazyDropdown({
    fetchItems, children, onSubmenuActivated: onLazyKeyActivated, onClose
}: LazyDropdownProps) {
    const [realItems, setRealItems] = useState<ItemType[]>([]);

    async function onSubmenuOpens(keys: React.Key[]) {
        const newItems = await onLazyKeyActivated(keys[keys.length - 1]);
        setRealItems(items => {
            console.log(`Appending ${newItems.length} items to key ${keys.join(' -> ')}`);
            return populateSubmenu(items, keys, newItems);
        });
    }

    return (
        <Dropdown
            trigger={['contextMenu']}
            menu={{
                items: realItems,
                onOpenChange(openKeys) {
                    if (openKeys.length === 0) return;
                    if (getSubmenuItems(realItems, openKeys)?.length ?? 0 > 0) return;
                    void onSubmenuOpens(openKeys);
                },
                onClick(e) {
                    e.domEvent.stopPropagation();
                }
            }}
            onOpenChange={(open) => {
                void (async () => {
                    if (open) {
                        setRealItems(await fetchItems());
                    }
                    else {
                        setRealItems([]);
                        onClose?.();
                    }
                })();
            }}
        >
            {children}
        </Dropdown>
    );
}

function getSubmenuItems(
    menu: ItemType[],
    path: React.Key[]
): ItemType[] | undefined {
    if (path.length === 0) {
        return menu;
    }

    const [currentKey, ...rest] = path;

    for (const item of menu) {
        if (!item) continue;

        // Found matching key at this level
        if (item.key === currentKey) {
            const submenu = item as SubMenuType;

            if (rest.length === 0) {
                return submenu.children;
            }

            if (!submenu.children) {
                return undefined;
            }

            return getSubmenuItems(submenu.children, rest);
        }

        // Not this item — search deeper
        const submenu = item as SubMenuType;
        if (submenu.children) {
            const result = getSubmenuItems(submenu.children, path);
            if (result !== undefined) {
                return result;
            }
        }
    }

    return undefined;
}

function populateSubmenu(
    menu: ItemType[],
    path: React.Key[],
    submenuItems: ItemType[]
): ItemType[] {
    if (path.length === 0) {
        return menu;
    }

    const [currentKey, ...rest] = path;
    let changed = false;

    const nextMenu = menu.map(item => {
        if (!item) return item;

        // Found the key at this level
        if (item.key === currentKey) {
            const submenu = item as SubMenuType;

            if (rest.length === 0) {
                changed = true;
                return {
                    ...submenu,
                    children: submenuItems,
                };
            }

            if (!submenu.children) {
                return item;
            }

            const updatedChildren = populateSubmenu(
                submenu.children,
                rest,
                submenuItems
            );

            if (updatedChildren !== submenu.children) {
                changed = true;
                return {
                    ...submenu,
                    children: updatedChildren,
                };
            }

            return item;
        }

        // Not this item → search deeper
        const submenu = item as SubMenuType;
        if (submenu.children) {
            const updatedChildren = populateSubmenu(
                submenu.children,
                path,
                submenuItems
            );

            if (updatedChildren !== submenu.children) {
                changed = true;
                return {
                    ...submenu,
                    children: updatedChildren,
                };
            }
        }

        return item;
    });

    return changed ? nextMenu : menu;
}
