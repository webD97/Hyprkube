import { Menu, MenuItem } from "@tauri-apps/api/menu";
import addHiddenGvk from "../../api/addHiddenGvk";
import addPinnedGvk from "../../api/addPinnedGvk";
import removePinnedGvk from "../../api/removePinnedGvk";
import { Gvk } from "../../model/k8s";

export async function createMenuForPinnedGvks(options: {
    clusterProfile: string,
    gvk: Gvk
}): Promise<Menu> {
    const unpin = MenuItem.new({
        text: "Unpin",
        action: () => {
            removePinnedGvk(options.clusterProfile, options.gvk)
                .catch(e => alert(JSON.stringify(e)));
        }
    });

    return Menu.new({ items: await Promise.all([unpin]) });
}

export async function createMenuForNormalGvks(options: {
    clusterProfile: string,
    gvk: Gvk
}): Promise<Menu> {
    const pinResource = MenuItem.new({
        text: "Pin",
        action: () => {
            addPinnedGvk(options.clusterProfile, options.gvk)
                .catch(e => alert(JSON.stringify(e)));
        }
    });

    const hideResource = MenuItem.new({
        text: "Hide",
        action: () => {
            addHiddenGvk(options.clusterProfile, options.gvk)
                .catch(e => alert(JSON.stringify(e)));
        }
    });

    return await Menu.new({ items: await Promise.all([pinResource, hideResource]) });
}
