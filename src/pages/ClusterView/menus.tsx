import { Menu, MenuItem, PredefinedMenuItem, Submenu } from "@tauri-apps/api/menu";
import { confirm } from '@tauri-apps/plugin-dialog';
import addHiddenGvk from "../../api/addHiddenGvk";
import addPinnedGvk from "../../api/addPinnedGvk";
import { deleteResource } from "../../api/deleteResource";
import listPodContainerNames from "../../api/listPodContainerNames";
import removePinnedGvk from "../../api/removePinnedGvk";
import LogPanel from "../../components/LogPanel";
import { Tab } from "../../components/TabView";
import { TabElement } from "../../components/TabView/hooks";
import HyprkubeTerminal from "../../components/Terminal";
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

export async function createMenuForResource(options: {
    namespace: string,
    name: string,
    gvk: Gvk,
    clientId: string,
    pushTab: (tab: TabElement) => void,
    onShowYaml: () => void,
    onSelectNamespace: (namespace: string) => void
}): Promise<Menu> {
    const {
        namespace, name,
        gvk, clientId,
        pushTab,
        onShowYaml,
        onSelectNamespace
    } = options;
    const itemPromises: Promise<MenuItem | PredefinedMenuItem>[] = [
        MenuItem.new({
            text: 'Show YAML',
            action: () => {
                onShowYaml();
            }
        }),
        MenuItem.new({
            text: 'Delete resource',
            action: () => {
                confirm(`This action cannot be reverted. Are you sure?`, { kind: 'warning', title: `Permanently delete resource?` })
                    .then(confirmed => {
                        if (confirmed) {
                            return deleteResource(clientId, gvk, namespace, name);
                        }
                    })
                    .catch(e => alert(JSON.stringify(e)));

            }
        }),
        PredefinedMenuItem.new({ item: 'Separator' }),
    ];

    if (namespace !== '') {
        itemPromises.push(
            MenuItem.new({
                text: 'Select namespace',
                action() {
                    onSelectNamespace(namespace)
                },
            })
        );
    }

    if (gvk.kind === "Pod") {
        const logItems: Promise<MenuItem>[] = [];
        const attachItems: Promise<MenuItem>[] = [];

        const containerNames = await listPodContainerNames(clientId, namespace, name);

        logItems.push(
            ...containerNames.map(containerName => (
                MenuItem.new({
                    text: containerName,
                    action: () => {
                        pushTab(
                            <Tab title={name} >
                                {
                                    () => (
                                        <LogPanel
                                            kubernetesClientId={clientId}
                                            namespace={namespace}
                                            name={name}
                                            container={containerName}
                                        />
                                    )
                                }
                            </Tab>
                        )
                    }
                })
            ))
        );

        attachItems.push(
            ...containerNames.map(containerName => (
                MenuItem.new({
                    text: containerName,
                    action: () => {
                        pushTab(
                            <Tab title={`Shell (${name})`}>
                                {
                                    () => (
                                        <HyprkubeTerminal
                                            clientId={clientId}
                                            podName={name}
                                            podNamespace={namespace}
                                            container={containerName}
                                        />
                                    )
                                }
                            </Tab>
                        );
                    }
                })
            ))
        );

        try {
            const logsSubmenu = Submenu.new({
                text: 'Show logs',
                items: await Promise.all(logItems)
            })

            const attachSubmenu = Submenu.new({
                text: 'Execute shell',
                items: await Promise.all(attachItems)
            });

            itemPromises.push(logsSubmenu, attachSubmenu);
        }
        catch (e) {
            throw new Error(e as string);
        }
    }

    try {
        const items = await Promise.all(itemPromises);
        return Menu.new({ items });
    }
    catch (e) {
        throw new Error(e as string);
    }
}
