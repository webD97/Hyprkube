import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

import { useCallback, useEffect, useMemo, useState } from 'react';

import EmojiHint from '../../components/EmojiHint';
import GvkList from '../../components/GvkList';
import LogPanel from '../../components/LogPanel';
import ResourceView from '../../components/ResourceView';
import TabView, { Tab } from '../../components/TabView';
import { useTabs } from '../../components/TabView/hooks';
import { useClusterDiscovery } from '../../hooks/useClusterDiscovery';
import useResourceWatch from '../../hooks/useResourceWatch';
import { Gvk } from '../../model/k8s';
import classes from './styles.module.css';
import { useSearchParams } from 'react-router-dom';
import useClusterNamespaces from '../../hooks/useClusterNamespaces';
import { deleteResource } from '../../api/deleteResource';
import listResourceViews, { type ResourceViewDef } from '../../api/listResourceViews';
import { confirm } from '@tauri-apps/plugin-dialog';
import { Menu, MenuItem, PredefinedMenuItem, Submenu } from '@tauri-apps/api/menu';
import HyprkubeTerminal from '../../components/Terminal';
import listPodContainerNames from '../../api/listPodContainerNames';
import listClusterProfiles, { ClusterProfile } from '../../api/listClusterProfiles';
import addPinnedGvk from '../../api/addPinnedGvk';
import removePinnedGvk from '../../api/removePinnedGvk';
import usePinnedGvks from '../../hooks/usePinnedGvks';
import useHiddenGvks from '../../hooks/useHiddenGvks';
import addHiddenGvk from '../../api/addHiddenGvk';
import { Editor } from '@monaco-editor/react';
import getResourceYaml from '../../api/getResourceYaml';

const namespace_gvk = { group: "", version: "v1", kind: "Namespace" };

const ClusterView: React.FC = () => {
    const [searchParams] = useSearchParams();
    const source = searchParams.get('source');
    const context = searchParams.get('context');

    const [availableViews, setAvailableViews] = useState<ResourceViewDef[]>([]);
    const [currentGvk, setCurrentGvk] = useState<Gvk>();
    const [clusterProfiles, setClusterProfiles] = useState<ClusterProfile[]>([]);
    const pinnedGvks = usePinnedGvks(clusterProfiles?.[0]?.[0]);
    const hiddenGvks = useHiddenGvks(clusterProfiles?.[0]?.[0]);
    const [selectedView, setSelectedView] = useState("");
    const { discovery, clientId, lastError, loading } = useClusterDiscovery(source, context);
    const namespaces = useClusterNamespaces(clientId, namespace_gvk);
    const [selectedNamespace, setSelectedNamespace] = useState('default');
    const [columnTitles, resources] = useResourceWatch(clientId, currentGvk, selectedView, selectedNamespace);

    const [tabs, activeTab, pushTab, removeTab, setActiveTab] = useTabs();

    useEffect(() => {
        listClusterProfiles()
            .then(profiles => {
                setClusterProfiles(profiles);
            })
            .catch(e => alert(JSON.stringify(e)));
    }, []);

    useEffect(() => {
        if (!clientId) return;
        if (!currentGvk) return;

        listResourceViews(clientId, currentGvk)
            .then(views => {
                setAvailableViews(views);

                if (views.length > 0) {
                    setSelectedView(views[0]);
                }
            })
            .catch(e => alert(JSON.stringify(e)));

    }, [currentGvk, clientId]);

    dayjs.extend(relativeTime);

    function findResourcePlural(gvk: Gvk) {
        return discovery.gvks[gvk.group]?.kinds.find(resource => resource.kind === gvk.kind)?.plural;
    }

    function findResourceScope(gvk: Gvk) {
        return discovery.gvks[gvk.group]?.kinds.find(resource => resource.kind === gvk.kind)?.scope;
    }

    const sortedPinnedGvks = useMemo(() => {
        return pinnedGvks.sort((a, b) => {
            if (a.kind < b.kind) {
                return -1;
            }
            if (a.kind > b.kind) {
                return 1;
            }

            return 0;
        });
    }, [pinnedGvks]);

    const yamlViewerFactory = useCallback(() => {
        if (clientId === undefined) {
            return () => undefined;
        }

        return (gvk: Gvk, resourceUID: string) => {
            const { namespace, name } = resources[resourceUID];

            getResourceYaml(clientId, gvk, namespace, name)
                .then((yaml) => {
                    pushTab(
                        <Tab title={`Show: ${name}`}>
                            {
                                () => (
                                    <Editor
                                        height="600px"
                                        width="100%"
                                        defaultLanguage="yaml"
                                        theme="vs-dark"
                                        options={{
                                            renderWhitespace: "all",
                                            readOnly: true,
                                        }}
                                        value={yaml}
                                    />
                                )
                            }
                        </Tab>
                    )
                })
                .catch(e => alert(JSON.stringify(e)));
        }
    }, [clientId, pushTab, resources]);

    return (
        <div className={classes.container}>
            <nav>
                <span>{context}</span>
                <h2>Pinned resources</h2>
                {
                    sortedPinnedGvks.length == 0
                        ? null
                        : (
                            <GvkList withGroupName
                                gvks={sortedPinnedGvks}
                                onResourceClicked={(gvk) => setCurrentGvk(gvk)}
                                onPinButtonClicked={(gvk) => void removePinnedGvk(clusterProfiles[0][0], gvk)}
                                onGvkRightClicked={async (gvk) => {
                                    const unpin = MenuItem.new({
                                        text: "Unpin",
                                        action: () => {
                                            removePinnedGvk(clusterProfiles[0][0], gvk)
                                                .catch(e => alert(JSON.stringify(e)));
                                        }
                                    });

                                    const menu = await Menu.new({ items: await Promise.all([unpin]) });

                                    await menu.popup();
                                }}
                            />
                        )
                }

                <h2>Builtin resources</h2>
                {
                    Object.values(discovery?.gvks || [])
                        .filter((group) => !group.isCrd)
                        .sort((groupA, groupB) => groupA.name.localeCompare(groupB.name))
                        .map(({ name: groupName, kinds }, idx) => {
                            const gvks = kinds
                                .map(({ kind, version }) => ({ group: groupName, version, kind }))
                                .filter(gvk => (
                                    !hiddenGvks.some(current => (
                                        current.group === gvk.group &&
                                        current.version === gvk.version &&
                                        current.kind === gvk.kind
                                    ))
                                ));

                            // Don't show groups where everything is hidden
                            if (gvks.length === 0) return;

                            return (
                                <details key={idx}>
                                    <summary>{groupName ? groupName : 'core'}</summary>
                                    <GvkList className={classes.gvkListIndented}
                                        gvks={gvks}
                                        onResourceClicked={(gvk) => setCurrentGvk(gvk)}
                                        onPinButtonClicked={(gvk) => void addPinnedGvk(clusterProfiles[0][0], gvk)}
                                        onGvkRightClicked={async (gvk) => {
                                            const unpin = MenuItem.new({
                                                text: "Pin",
                                                action: () => {
                                                    addPinnedGvk(clusterProfiles[0][0], gvk)
                                                        .catch(e => alert(JSON.stringify(e)));
                                                }
                                            });

                                            const hide = MenuItem.new({
                                                text: "Hide",
                                                action: () => {
                                                    addHiddenGvk(clusterProfiles[0][0], gvk)
                                                        .catch(e => alert(JSON.stringify(e)));
                                                }
                                            });

                                            const menu = await Menu.new({ items: await Promise.all([unpin, hide]) });

                                            await menu.popup();
                                        }}
                                    />
                                </details>
                            );
                        })
                }

                <h2>Custom resources</h2>
                {
                    Object.values(discovery?.gvks || [])
                        .filter((group) => group.isCrd)
                        .sort((groupA, groupB) => groupA.name.localeCompare(groupB.name))
                        .map(({ name: groupName, kinds }) => {
                            const gvks = kinds
                                .map(({ kind, version }) => ({ group: groupName, version, kind }))
                                .filter(gvk => (
                                    !hiddenGvks.some(current => (
                                        current.group === gvk.group &&
                                        current.version === gvk.version &&
                                        current.kind === gvk.kind
                                    ))
                                ));

                            // Don't show groups where everything is hidden
                            if (gvks.length === 0) return;

                            return (
                                <details key={groupName}>
                                    <summary>{groupName ? groupName : 'core'}</summary>
                                    <GvkList className={classes.gvkListIndented}
                                        gvks={gvks}
                                        onResourceClicked={(gvk) => setCurrentGvk(gvk)}
                                        onPinButtonClicked={(gvk) => void addPinnedGvk(clusterProfiles[0][0], gvk)}
                                        onGvkRightClicked={async (gvk) => {
                                            const unpin = MenuItem.new({
                                                text: "Pin",
                                                action: () => {
                                                    addPinnedGvk(clusterProfiles[0][0], gvk)
                                                        .catch(e => alert(JSON.stringify(e)));
                                                }
                                            });

                                            const hide = MenuItem.new({
                                                text: "Hide",
                                                action: () => {
                                                    addHiddenGvk(clusterProfiles[0][0], gvk)
                                                        .catch(e => alert(JSON.stringify(e)));
                                                }
                                            });

                                            const menu = await Menu.new({ items: await Promise.all([unpin, hide]) });

                                            await menu.popup();
                                        }}
                                    />
                                </details>
                            );
                        })
                }
            </nav>
            <section className={classes.bottomPanel}>
                <TabView
                    activeTab={activeTab}
                    onCloseClicked={(idx) => removeTab(idx)}
                    setActiveTab={setActiveTab}
                >
                    {tabs}
                </TabView>
            </section>
            <section className={classes.mainArea}>
                {
                    loading
                        ? <EmojiHint emoji="⏳">Loading...</EmojiHint>
                        : lastError !== undefined
                            ? <EmojiHint emoji="💩"><span style={{ color: 'red' }}>{lastError}</span></EmojiHint>
                            : currentGvk === undefined
                                ? <EmojiHint emoji="🔍">Select a resource to get started.</EmojiHint>
                                : (
                                    <>
                                        <div className={classes.topBar}>
                                            <h2>{findResourcePlural(currentGvk)}</h2>
                                            <select value={selectedView} onChange={(e) => setSelectedView(e.target.value)}>
                                                {
                                                    availableViews.map(view => (
                                                        <option key={view}>{view}</option>
                                                    ))
                                                }
                                            </select>
                                            {
                                                findResourceScope(currentGvk) === 'cluster'
                                                    ? null
                                                    : (
                                                        <select value={selectedNamespace} onChange={(e) => setSelectedNamespace(e.target.value)}>
                                                            <option label="(All namespaces)"></option>
                                                            {
                                                                Object.values(namespaces).map(namespace => (
                                                                    <option key={namespace}>{namespace}</option>
                                                                ))
                                                            }
                                                        </select>
                                                    )
                                            }
                                        </div>
                                        <ResourceView
                                            resourceNamePlural={findResourcePlural(currentGvk)}
                                            gvk={currentGvk}
                                            namespace={selectedNamespace}
                                            columnTitles={columnTitles || []}
                                            resourceData={resources}
                                            onResourceClicked={yamlViewerFactory()}
                                            onResourceContextMenu={async (gvk, resourceUID) => {
                                                const { namespace, name } = resources[resourceUID];

                                                const itemPromises: Promise<MenuItem | PredefinedMenuItem>[] = [
                                                    MenuItem.new({
                                                        text: 'Show YAML',
                                                        action: () => yamlViewerFactory()(gvk, resourceUID)
                                                    }),
                                                    MenuItem.new({
                                                        text: 'Delete resource',
                                                        action: () => {
                                                            const { namespace, name } = resources[resourceUID];

                                                            confirm(`This action cannot be reverted. Are you sure?`, { kind: 'warning', title: `Permanently delete resource?` })
                                                                .then(confirmed => {
                                                                    if (confirmed) {
                                                                        return deleteResource(clientId!, currentGvk, namespace, name);
                                                                    }
                                                                })
                                                                .catch(e => alert(JSON.stringify(e)));

                                                        }
                                                    }),
                                                    PredefinedMenuItem.new({ item: 'Separator' }),
                                                ];

                                                if (currentGvk.kind === "Pod") {
                                                    const logItems: Promise<MenuItem>[] = [];
                                                    const attachItems: Promise<MenuItem>[] = [];

                                                    const containerNames = await listPodContainerNames(clientId!, namespace, name);

                                                    logItems.push(
                                                        ...containerNames.map(containerName => (
                                                            MenuItem.new({
                                                                text: containerName,
                                                                action: () => {
                                                                    pushTab(
                                                                        <Tab title={name}>
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
                                                                                        clientId={clientId!}
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
                                            }}
                                        />
                                    </>
                                )
                }
            </section>
        </div>
    )
}

export default ClusterView;
