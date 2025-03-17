import { Menu, MenuItem } from "@tauri-apps/api/menu";
import addHiddenGvk from "../../api/addHiddenGvk";
import addPinnedGvk from "../../api/addPinnedGvk";
import removePinnedGvk from "../../api/removePinnedGvk";
import { Gvk } from "../../model/k8s";

export async function createMenuForPinnedGvks(options: {
    clusterProfile: string,
    gvk: Gvk,
    openInNewTab: (gvk: Gvk) => void
}): Promise<Menu> {
    const items: Promise<MenuItem>[] = [];

    items.push(
        MenuItem.new({
            text: "Open in new tab",
            action: () => {
                options.openInNewTab(options.gvk);
            }
        })
    );

    items.push(
        MenuItem.new({
            text: "Unpin",
            action: () => {
                removePinnedGvk(options.clusterProfile, options.gvk)
                    .catch(e => alert(JSON.stringify(e)));
            }
        })
    );

    return Menu.new({ items: await Promise.all(items) });
}

export async function createMenuForNormalGvks(options: {
    clusterProfile: string,
    gvk: Gvk,
    openInNewTab: (gvk: Gvk) => void
}): Promise<Menu> {
    const items: Promise<MenuItem>[] = [];

    items.push(
        MenuItem.new({
            text: "Open in new tab",
            action: () => {
                options.openInNewTab(options.gvk);
            }
        })
    );

    items.push(
        MenuItem.new({
            text: "Pin",
            action: () => {
                addPinnedGvk(options.clusterProfile, options.gvk)
                    .catch(e => alert(JSON.stringify(e)));
            }
        })
    );

    items.push(
        MenuItem.new({
            text: "Hide",
            action: () => {
                addHiddenGvk(options.clusterProfile, options.gvk)
                    .catch(e => alert(JSON.stringify(e)));
            }
        })
    );

    return await Menu.new({ items: await Promise.all(items) });
}
